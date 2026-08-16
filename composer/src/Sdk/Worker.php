<?php

declare(strict_types=1);

namespace Mago\Sdk;

use Mago\Sdk\Analyzer\AfterAnalysisContext;
use Mago\Sdk\Analyzer\AfterFileAnalysisContext;
use Mago\Sdk\Analyzer\BeforeAnalysisContext;
use Mago\Sdk\Analyzer\CallableSignatureProvider;
use Mago\Sdk\Analyzer\CallableSignatureProviderContext;
use Mago\Sdk\Analyzer\ClassInitializerProviderContext;
use Mago\Sdk\Analyzer\Codebase;
use Mago\Sdk\Analyzer\FileAnalysis;
use Mago\Sdk\Analyzer\InitializationContext;
use Mago\Sdk\Analyzer\InvocationKind;
use Mago\Sdk\Analyzer\IssueFilterContext;
use Mago\Sdk\Analyzer\IssueFilterDecision;
use Mago\Sdk\Analyzer\NodeAnalysisContext;
use Mago\Sdk\Analyzer\PluginRegistry as AnalyzerPluginRegistry;
use Mago\Sdk\Analyzer\ProjectAnalysis;
use Mago\Sdk\Analyzer\PropertyInitializationProviderContext;
use Mago\Sdk\Analyzer\PropertyTypeProviderContext;
use Mago\Sdk\Analyzer\ReturnTypeProviderContext;
use Mago\Sdk\Analyzer\TypeComparator;
use Mago\Sdk\Exception\CancelledException;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Analyzer\DefinitionName;
use Mago\Sdk\Internal\Analyzer\MetadataCache;
use Mago\Sdk\Internal\Analyzer\Protocol as AnalyzerProtocol;
use Mago\Sdk\Internal\Analyzer\RegisteredClassInitializerProvider;
use Mago\Sdk\Internal\Analyzer\RegisteredFunctionReturnTypeProvider;
use Mago\Sdk\Internal\Analyzer\RegisteredIssueFilterHook;
use Mago\Sdk\Internal\Analyzer\RegisteredMethodReturnTypeProvider;
use Mago\Sdk\Internal\Analyzer\RegisteredNodeAnalysisHook;
use Mago\Sdk\Internal\Analyzer\RegisteredPlugin;
use Mago\Sdk\Internal\Analyzer\RegisteredPropertyInitializationProvider;
use Mago\Sdk\Internal\Analyzer\RegisteredPropertyTypeProvider;
use Mago\Sdk\Internal\Analyzer\ReportedIssue;
use Mago\Sdk\Internal\HostClient;
use Mago\Sdk\Internal\Io\InputTransport;
use Mago\Sdk\Internal\Io\ResourceReader;
use Mago\Sdk\Internal\Io\ResourceWriter;
use Mago\Sdk\Internal\Linter\Protocol as LinterProtocol;
use Mago\Sdk\Internal\Linter\RegisteredRule;
use Mago\Sdk\Internal\Protocol\Frame;
use Mago\Sdk\Internal\Protocol\FrameCodec;
use Mago\Sdk\Internal\Protocol\FrameKind;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\SignalCancellationToken;
use Mago\Sdk\Internal\Worker\Protocol as WorkerProtocol;
use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Syntax\NodeKind;
use Revolt\EventLoop;
use Throwable;

use function array_key_exists;
use function array_values;
use function count;
use function defined;
use function fclose;
use function fwrite;
use function in_array;
use function is_array;
use function ob_end_flush;
use function ob_get_level;
use function ob_start;
use function strncmp;
use function strtolower;

use const STDERR;
use const STDIN;
use const STDOUT;

/**
 * Persistent worker process serving one or more Mago extensions.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 * @mago-expect lint:too-many-methods
 * @mago-expect lint:too-many-properties
 */
final class Worker
{
    /**
     * @var non-empty-list<Extension>
     */
    private readonly array $extensions;

    /**
     * @var list<RegisteredRule>
     */
    private readonly array $rules;

    /** @var list<RegisteredPlugin> */
    private readonly array $analyzerPlugins;

    /** @var list<int<0, max>> */
    private readonly array $workerReducerIndices;

    /** @var list<RegisteredFunctionReturnTypeProvider> */
    private readonly array $functionReturnTypeProviders;

    /** @var list<RegisteredMethodReturnTypeProvider> */
    private readonly array $methodReturnTypeProviders;

    /** @var list<RegisteredPropertyTypeProvider> */
    private readonly array $propertyTypeProviders;

    /** @var list<RegisteredPropertyInitializationProvider> */
    private readonly array $propertyInitializationProviders;

    /** @var list<RegisteredClassInitializerProvider> */
    private readonly array $classInitializerProviders;

    /** @var list<RegisteredIssueFilterHook> */
    private readonly array $issueFilterHooks;

    /**
     * @var list<int<0, 65535>>
     */
    private array $activeRuleIndices = [];

    /**
     * @var array<string, list<RegisteredRule>>
     */
    private array $activeRulesByNodeKind = [];

    /**
     * @var list<NodeKind>
     */
    private array $nodeKinds = [];

    private PHPVersion $phpVersion;

    private ?MetadataCache $metadataCache = null;

    /**
     * @mago-expect lint:halstead
     */
    public function __construct(Extension $extension, Extension ...$additionalExtensions)
    {
        $extensions = [$extension, ...array_values($additionalExtensions)];
        $extensionIdentifiers = [];
        $ruleCodes = [];
        $rules = [];
        $pluginSelectors = [];
        $registeredPlugins = [];
        $functionProviders = [];
        $methodProviders = [];
        $propertyProviders = [];
        $propertyInitializationProviders = [];
        $classInitializerProviders = [];
        $issueFilterHooks = [];
        $nodeAnalysisHooks = [];
        $workerReducerIndices = [];
        foreach ($extensions as $extensionIndex => $registeredExtension) {
            $normalizedExtensionIdentifier = strtolower($registeredExtension->identifier);
            if (array_key_exists($normalizedExtensionIdentifier, $extensionIdentifiers)) {
                throw new InvalidArgumentException(
                    "Extension `{$registeredExtension->identifier}` is registered more than once.",
                );
            }

            $extensionIdentifiers[$normalizedExtensionIdentifier] = true;
            if ($registeredExtension->workerReducer !== null) {
                $workerReducerIndices[] = $extensionIndex;
            }

            foreach ($registeredExtension->linterRules as $rule) {
                $definition = $rule->getDefinition();
                if (array_key_exists($definition->code, $ruleCodes)) {
                    throw new InvalidArgumentException(
                        "Linter rule `{$definition->code}` is registered by more than one extension.",
                    );
                }

                $ruleCodes[$definition->code] = true;
                $ruleIndex = count($rules);
                if ($ruleIndex > 65_535) {
                    throw new InvalidArgumentException('A worker cannot register more than 65,536 linter rules.');
                }

                $rules[] = new RegisteredRule($ruleIndex, $rule, $definition);
            }

            foreach ($registeredExtension->analyzerPlugins as $plugin) {
                $definition = $plugin->getDefinition();
                foreach ([$definition->identifier, ...$definition->aliases] as $selector) {
                    $normalizedSelector = strtolower($selector);
                    if (array_key_exists($normalizedSelector, $pluginSelectors)) {
                        $first = $pluginSelectors[$normalizedSelector];
                        throw new InvalidArgumentException(
                            "Analyzer plugin selector `{$selector}` is shared by plugins `{$first}` and `{$definition->identifier}`.",
                        );
                    }

                    $pluginSelectors[$normalizedSelector] = $definition->identifier;
                }
                $registry = new AnalyzerPluginRegistry();
                $plugin->register($registry);
                $registeredFunctionProviders = [];
                foreach ($registry->getFunctionReturnTypeProviders() as $provider) {
                    $targets = $provider->getTargets();
                    if ($targets === []) {
                        throw new InvalidArgumentException(
                            "A function return-type provider in `{$definition->identifier}` has no targets.",
                        );
                    }

                    $providerIndex = count($functionProviders);
                    if ($providerIndex > 65_535) {
                        throw new InvalidArgumentException(
                            'A worker cannot register more than 65,536 function return-type providers.',
                        );
                    }

                    $registeredProvider = new RegisteredFunctionReturnTypeProvider(
                        $providerIndex,
                        $definition->identifier,
                        $provider,
                        $targets,
                    );
                    $functionProviders[] = $registeredProvider;
                    $registeredFunctionProviders[] = $registeredProvider;
                }

                $registeredMethodProviders = [];
                foreach ($registry->getMethodReturnTypeProviders() as $provider) {
                    $targets = $provider->getTargets();
                    if ($targets === []) {
                        throw new InvalidArgumentException(
                            "A method return-type provider in `{$definition->identifier}` has no targets.",
                        );
                    }

                    $providerIndex = count($methodProviders);
                    if ($providerIndex > 65_535) {
                        throw new InvalidArgumentException(
                            'A worker cannot register more than 65,536 method return-type providers.',
                        );
                    }

                    $registeredProvider = new RegisteredMethodReturnTypeProvider(
                        $providerIndex,
                        $definition->identifier,
                        $provider,
                        $targets,
                    );
                    $methodProviders[] = $registeredProvider;
                    $registeredMethodProviders[] = $registeredProvider;
                }

                $registeredPropertyProviders = [];
                foreach ($registry->getPropertyTypeProviders() as $provider) {
                    $targets = $provider->getTargets();
                    if ($targets === []) {
                        throw new InvalidArgumentException(
                            "A property type provider in `{$definition->identifier}` has no targets.",
                        );
                    }

                    $providerIndex = count($propertyProviders);
                    if ($providerIndex > 65_535) {
                        throw new InvalidArgumentException(
                            'A worker cannot register more than 65,536 property type providers.',
                        );
                    }

                    $registeredProvider = new RegisteredPropertyTypeProvider(
                        $providerIndex,
                        $definition->identifier,
                        $provider,
                        $targets,
                    );
                    $propertyProviders[] = $registeredProvider;
                    $registeredPropertyProviders[] = $registeredProvider;
                }

                $registeredPropertyInitializationProviders = [];
                foreach ($registry->getPropertyInitializationProviders() as $provider) {
                    $targets = $provider->getTargets();
                    if ($targets === []) {
                        throw new InvalidArgumentException(
                            "A property initialization provider in `{$definition->identifier}` has no targets.",
                        );
                    }

                    $providerIndex = count($propertyInitializationProviders);
                    if ($providerIndex > 65_535) {
                        throw new InvalidArgumentException(
                            'A worker cannot register more than 65,536 property initialization providers.',
                        );
                    }

                    $registeredProvider = new RegisteredPropertyInitializationProvider(
                        $providerIndex,
                        $definition->identifier,
                        $provider,
                        $targets,
                    );
                    $propertyInitializationProviders[] = $registeredProvider;
                    $registeredPropertyInitializationProviders[] = $registeredProvider;
                }

                $registeredClassInitializerProviders = [];
                foreach ($registry->getClassInitializerProviders() as $provider) {
                    $targets = $provider->getTargets();
                    if ($targets === []) {
                        throw new InvalidArgumentException(
                            "A class initializer provider in `{$definition->identifier}` has no targets.",
                        );
                    }

                    $providerIndex = count($classInitializerProviders);
                    if ($providerIndex > 65_535) {
                        throw new InvalidArgumentException(
                            'A worker cannot register more than 65,536 class initializer providers.',
                        );
                    }

                    $registeredProvider = new RegisteredClassInitializerProvider(
                        $providerIndex,
                        $definition->identifier,
                        $provider,
                        $targets,
                    );
                    $classInitializerProviders[] = $registeredProvider;
                    $registeredClassInitializerProviders[] = $registeredProvider;
                }

                $registeredIssueFilterHooks = [];
                foreach ($registry->getIssueFilterHooks() as $hook) {
                    $codes = $hook->getCodes();
                    if ($codes === []) {
                        throw new InvalidArgumentException(
                            "An issue-filter hook in `{$definition->identifier}` has no target codes.",
                        );
                    }
                    $seenCodes = [];
                    foreach ($codes as $code) {
                        if (array_key_exists($code, $seenCodes)) {
                            throw new InvalidArgumentException(
                                "An issue-filter hook in `{$definition->identifier}` targets issue code `{$code}` more than once.",
                            );
                        }
                        $seenCodes[$code] = true;
                    }

                    $hookIndex = count($issueFilterHooks);
                    if ($hookIndex > 65_535) {
                        throw new InvalidArgumentException(
                            'A worker cannot register more than 65,536 analyzer issue-filter hooks.',
                        );
                    }

                    $registeredHook = new RegisteredIssueFilterHook($hookIndex, $definition->identifier, $hook, $codes);
                    $issueFilterHooks[] = $registeredHook;
                    $registeredIssueFilterHooks[] = $registeredHook;
                }

                $registeredNodeAnalysisHooks = [];
                $registeredNodeAnalysisHooksByNodeKind = [];
                foreach ($registry->getNodeAnalysisHooks() as $hook) {
                    $targets = $hook->getTargets();
                    if ($targets === []) {
                        throw new InvalidArgumentException(
                            "A node-analysis hook in `{$definition->identifier}` has no targets.",
                        );
                    }

                    $seenTargets = [];
                    foreach ($targets as $target) {
                        if (array_key_exists($target->value, $seenTargets)) {
                            throw new InvalidArgumentException(
                                "A node-analysis hook in `{$definition->identifier}` targets `{$target->value}` more than once.",
                            );
                        }
                        $seenTargets[$target->value] = true;
                    }

                    $requirements = $hook->getRequirements();
                    $seenRequirements = [];
                    foreach ($requirements as $requirement) {
                        if (array_key_exists($requirement->name, $seenRequirements)) {
                            throw new InvalidArgumentException(
                                "A node-analysis hook in `{$definition->identifier}` requests `{$requirement->name}` more than once.",
                            );
                        }
                        $seenRequirements[$requirement->name] = true;
                    }

                    $hookIndex = count($nodeAnalysisHooks);
                    if ($hookIndex > 65_535) {
                        throw new InvalidArgumentException(
                            'A worker cannot register more than 65,536 analyzer node-analysis hooks.',
                        );
                    }

                    $registeredHook = new RegisteredNodeAnalysisHook(
                        $hookIndex,
                        $definition->identifier,
                        $hook,
                        $targets,
                        $requirements,
                    );
                    $nodeAnalysisHooks[] = $registeredHook;
                    $registeredNodeAnalysisHooks[] = $registeredHook;
                    foreach ($targets as $target) {
                        $registeredNodeAnalysisHooksByNodeKind[$target->value][] = $registeredHook;
                    }
                }

                $registeredPlugins[] = new RegisteredPlugin(
                    count($registeredPlugins),
                    $registeredExtension->identifier,
                    $plugin,
                    $definition,
                    $registeredFunctionProviders,
                    $registeredMethodProviders,
                    $registeredPropertyProviders,
                    $registeredPropertyInitializationProviders,
                    $registeredClassInitializerProviders,
                    $registeredIssueFilterHooks,
                    $registry->getInitializationHooks(),
                    $registry->getBeforeAnalysisHooks(),
                    $registry->getAfterFileAnalysisHooks(),
                    $registeredNodeAnalysisHooks,
                    $registeredNodeAnalysisHooksByNodeKind,
                    $registry->getAfterAnalysisHooks(),
                    $registry->shouldMemoizeProviders(),
                );
            }
        }

        $this->extensions = $extensions;
        $this->rules = $rules;
        $this->analyzerPlugins = $registeredPlugins;
        $this->workerReducerIndices = $workerReducerIndices;
        $this->functionReturnTypeProviders = $functionProviders;
        $this->methodReturnTypeProviders = $methodProviders;
        $this->propertyTypeProviders = $propertyProviders;
        $this->propertyInitializationProviders = $propertyInitializationProviders;
        $this->classInitializerProviders = $classInitializerProviders;
        $this->issueFilterHooks = $issueFilterHooks;
    }

    /**
     * Serve requests until Mago closes the input stream or requests shutdown.
     *
     * Each request runs in its own Fiber. CPU-bound callbacks remain sequential,
     * while callbacks using cooperative Revolt I/O may interleave.
     *
     * @param resource|null $input
     * @param resource|null $output
     * @param int $maximumPayloadSize Maximum accepted frame payload in bytes.
     * @mago-expect lint:halstead
     */
    public function run(
        mixed $input = null,
        mixed $output = null,
        int $maximumPayloadSize = FrameCodec::DEFAULT_MAXIMUM_PAYLOAD_SIZE,
    ): void {
        $codec = new FrameCodec($maximumPayloadSize);
        $connectedInput = $input === null ? InputTransport::connect() : null;
        $reader = new ResourceReader($input ?? $connectedInput ?? STDIN);
        $writer = new ResourceWriter($output ?? STDOUT);
        $host = new HostClient($codec, $writer);

        /** @var array<int<0, max>, SignalCancellationToken> $requests */
        $requests = [];
        $outputBufferLevel = $this->captureAccidentalOutput();
        try {
            while (true) {
                $frame = $codec->read($reader);
                if ($frame === null) {
                    $this->cancelRequests($requests);
                    $host->fail(new ProtocolException('Mago closed the worker input stream.'));
                    return;
                }

                if ($frame->kind === FrameKind::Shutdown) {
                    $this->cancelRequests($requests);
                    $host->fail(new ProtocolException('Mago requested worker shutdown.'));
                    return;
                }

                if ($frame->kind === FrameKind::Cancel) {
                    if (array_key_exists($frame->id, $requests)) {
                        $requests[$frame->id]->cancel();
                    }

                    continue;
                }

                if ($host->accept($frame)) {
                    continue;
                }

                if ($frame->kind !== FrameKind::Request) {
                    throw new ProtocolException("Unexpected {$frame->kind->name} frame from Mago.");
                }

                if ($frame->flags !== 0 || $frame->parentId !== 0 || $frame->id === 0) {
                    throw new ProtocolException('A top-level Mago request contains invalid frame metadata.');
                }

                if (array_key_exists($frame->id, $requests)) {
                    throw new ProtocolException("Mago reused in-flight request identifier {$frame->id}.");
                }

                $cancellation = new SignalCancellationToken();
                $requests[$frame->id] = $cancellation;
                EventLoop::queue(function () use ($frame, $cancellation, $codec, $writer, $host, &$requests): void {
                    try {
                        $payload = $this->handleRequest($frame->payload, $frame->id, $host, $cancellation);
                        $response = $codec->encodeResponse($frame->id, $payload);
                    } catch (CancelledException) {
                        return;
                    } catch (Throwable $throwable) {
                        $this->writeThrowable($throwable);
                        $response = $codec->encodeResponse($frame->id, $throwable->getMessage(), Frame::ERROR_FLAG);
                    } finally {
                        unset($requests[$frame->id]);
                    }

                    $writer->write($response);
                });
            }
        } finally {
            $reader->close();
            $writer->close();
            if ($connectedInput !== null) {
                fclose($connectedInput);
            }
            while (ob_get_level() > $outputBufferLevel) {
                ob_end_flush();
            }
        }
    }

    /**
     * @param positive-int $requestId
     */
    private function handleRequest(
        string $payload,
        int $requestId,
        HostClient $host,
        CancellationTokenInterface $cancellation,
    ): string {
        if (strncmp($payload, 'MLNT', 4) === 0) {
            return $this->handleLinterRequest($payload, $cancellation);
        }

        if (strncmp($payload, 'MANA', 4) === 0) {
            return $this->handleAnalyzerRequest($payload, $requestId, $host, $cancellation);
        }

        if (strncmp($payload, 'MEXT', 4) === 0) {
            return $this->handleWorkerRequest($payload, $cancellation);
        }

        throw new ProtocolException('Unknown Mago extension capability protocol.');
    }

    private function handleWorkerRequest(string $payload, CancellationTokenInterface $cancellation): string
    {
        [$kind, $reader] = WorkerProtocol::readRequest($payload);
        if ($kind === WorkerProtocol::COLLECT_REQUEST) {
            $reader->finish();
            $payloads = [];
            foreach ($this->workerReducerIndices as $extensionIndex) {
                $cancellation->throwIfCancelled();
                $extension = $this->extensions[$extensionIndex];
                try {
                    $payloads[$extensionIndex] = $extension->workerReducer?->collect() ?? '';
                } catch (CancelledException $exception) {
                    throw $exception;
                } catch (Throwable $throwable) {
                    throw new ProtocolException(
                        "Worker reducer for extension `{$extension->identifier}` failed to collect: {$throwable->getMessage()}",
                        0,
                        $throwable,
                    );
                }
            }

            return WorkerProtocol::writeCollectResponse($payloads);
        }

        $payloadsByExtension = WorkerProtocol::readReduceRequest($reader, $this->workerReducerIndices);
        foreach ($this->workerReducerIndices as $extensionIndex) {
            $cancellation->throwIfCancelled();
            $extension = $this->extensions[$extensionIndex];
            try {
                $extension->workerReducer?->reduce(
                    new WorkerReductionContext($payloadsByExtension[$extensionIndex], $cancellation),
                );
            } catch (CancelledException $exception) {
                throw $exception;
            } catch (Throwable $throwable) {
                throw new ProtocolException(
                    "Worker reducer for extension `{$extension->identifier}` failed to reduce: {$throwable->getMessage()}",
                    0,
                    $throwable,
                );
            }
        }

        return WorkerProtocol::writeReduceResponse();
    }

    private function handleLinterRequest(string $payload, CancellationTokenInterface $cancellation): string
    {
        [$kind, $reader] = LinterProtocol::readRequest($payload);
        if ($kind === LinterProtocol::DESCRIBE_REQUEST) {
            [$this->phpVersion, $this->nodeKinds] = LinterProtocol::readDescribeRequest($reader);

            return LinterProtocol::writeDescribeResponse($this->extensions);
        }

        if ($kind !== LinterProtocol::LINT_FILE_REQUEST) {
            throw new ProtocolException("Unknown linter request kind {$kind}.");
        }

        $request = LinterProtocol::readLintRequest($reader, $this->phpVersion, $this->nodeKinds);
        if ($request->activeRules !== $this->activeRuleIndices) {
            $activeRulesByNodeKind = [];
            foreach ($request->activeRules as $ruleIndex) {
                $registeredRule = $this->rules[$ruleIndex] ?? null;
                if ($registeredRule === null) {
                    throw new ProtocolException("Mago requested unregistered linter rule index {$ruleIndex}.");
                }

                foreach ($registeredRule->definition->targets as $target) {
                    $activeRulesByNodeKind[$target->value][] = $registeredRule;
                }
            }

            $this->activeRuleIndices = $request->activeRules;
            $this->activeRulesByNodeKind = $activeRulesByNodeKind;
        }

        $reportedIssues = [];
        $targetIndex = 0;
        foreach ($request->file->getTargetNodes() as $node) {
            if (($targetIndex++ & 63) === 0) {
                $cancellation->throwIfCancelled();
            }

            $context = new LintContext($request->file, $node, $cancellation);
            foreach ($this->activeRulesByNodeKind[$node->kind->value] ?? [] as $registeredRule) {
                try {
                    $registeredRule->rule->lint($context);
                } catch (Throwable $throwable) {
                    $code = $registeredRule->definition->code;
                    throw new ProtocolException(
                        "Linter rule `{$code}` failed: {$throwable->getMessage()}",
                        0,
                        $throwable,
                    );
                }

                foreach ($context->issues as $issue) {
                    $reportedIssues[] = $registeredRule->index;
                    $reportedIssues[] = $issue;
                }

                $context->issues = [];
            }
        }

        return LinterProtocol::writeLintResponse($reportedIssues);
    }

    /**
     * @param positive-int $requestId
     *
     * @mago-expect lint:halstead
     */
    private function handleAnalyzerRequest(
        string $payload,
        int $requestId,
        HostClient $host,
        CancellationTokenInterface $cancellation,
    ): string {
        [$kind, $reader] = AnalyzerProtocol::readRequest($payload);
        if ($kind === AnalyzerProtocol::DESCRIBE_REQUEST) {
            [$this->phpVersion, $this->nodeKinds] = AnalyzerProtocol::readDescribeRequest($reader);

            return AnalyzerProtocol::writeDescribeResponse($this->extensions, $this->analyzerPlugins);
        }

        if ($kind === AnalyzerProtocol::INITIALIZE_REQUEST) {
            return $this->handleAnalyzerInitialization($reader, $cancellation);
        }

        if (
            $kind === AnalyzerProtocol::BEFORE_ANALYSIS_REQUEST
            || $kind === AnalyzerProtocol::AFTER_FILE_ANALYSIS_REQUEST
            || $kind === AnalyzerProtocol::AFTER_ANALYSIS_REQUEST
            || $kind === AnalyzerProtocol::AFTER_FILE_ANALYSIS_BATCH_REQUEST
        ) {
            return $this->handleAnalyzerLifecycleRequest($kind, $reader, $requestId, $host, $cancellation);
        }

        if ($kind === AnalyzerProtocol::PROPERTY_TYPE_REQUEST) {
            $request = AnalyzerProtocol::readPropertyTypeRequest($reader);
            if ($this->metadataCache === null || $this->metadataCache->generation !== $request->generation) {
                $this->metadataCache = new MetadataCache($request->generation);
            }

            $codebase = new Codebase($host, $requestId, $cancellation, $this->metadataCache);
            foreach ($request->providerIndices as $providerIndex) {
                $registered = $this->propertyTypeProviders[$providerIndex] ?? null;
                if ($registered === null) {
                    throw new ProtocolException(
                        "Mago requested unregistered property type provider index {$providerIndex}.",
                    );
                }

                $cancellation->throwIfCancelled();
                try {
                    $type = $registered->provider->getPropertyType(
                        new PropertyTypeProviderContext(
                            $this->phpVersion,
                            $codebase,
                            $request->access,
                            new TypeComparator($host, $requestId, $cancellation, $this->metadataCache),
                            $cancellation,
                        ),
                    );
                } catch (Throwable $throwable) {
                    throw new ProtocolException(
                        "Property type provider in `{$registered->plugin}` failed: {$throwable->getMessage()}",
                        0,
                        $throwable,
                    );
                }

                if ($type !== null) {
                    return AnalyzerProtocol::writePropertyTypeResponse($type);
                }
            }

            return AnalyzerProtocol::writePropertyTypeResponse(null);
        }

        if ($kind === AnalyzerProtocol::PROPERTY_INITIALIZATION_REQUEST) {
            $request = AnalyzerProtocol::readPropertyInitializationRequest($reader);
            if ($this->metadataCache === null || $this->metadataCache->generation !== $request->generation) {
                $this->metadataCache = new MetadataCache($request->generation);
            }

            $codebase = new Codebase($host, $requestId, $cancellation, $this->metadataCache);
            foreach ($request->providerIndices as $providerIndex) {
                $registered = $this->propertyInitializationProviders[$providerIndex] ?? null;
                if ($registered === null) {
                    throw new ProtocolException(
                        "Mago requested unregistered property initialization provider index {$providerIndex}.",
                    );
                }

                $cancellation->throwIfCancelled();
                try {
                    $initialized = $registered->provider->isPropertyInitialized(
                        new PropertyInitializationProviderContext(
                            $this->phpVersion,
                            $codebase,
                            $request->declaringClass,
                            $request->property,
                            new TypeComparator($host, $requestId, $cancellation, $this->metadataCache),
                            $cancellation,
                        ),
                    );
                } catch (Throwable $throwable) {
                    throw new ProtocolException(
                        "Property initialization provider in `{$registered->plugin}` failed: {$throwable->getMessage()}",
                        0,
                        $throwable,
                    );
                }

                if ($initialized) {
                    return AnalyzerProtocol::writePropertyInitializationResponse(true);
                }
            }

            return AnalyzerProtocol::writePropertyInitializationResponse(false);
        }

        if ($kind === AnalyzerProtocol::CLASS_INITIALIZER_REQUEST) {
            $request = AnalyzerProtocol::readClassInitializerRequest($reader);
            if ($this->metadataCache === null || $this->metadataCache->generation !== $request->generation) {
                $this->metadataCache = new MetadataCache($request->generation);
            }

            $codebase = new Codebase($host, $requestId, $cancellation, $this->metadataCache);
            $initializers = [];
            foreach ($request->providerIndices as $providerIndex) {
                $registered = $this->classInitializerProviders[$providerIndex] ?? null;
                if ($registered === null) {
                    throw new ProtocolException(
                        "Mago requested unregistered class initializer provider index {$providerIndex}.",
                    );
                }

                $cancellation->throwIfCancelled();
                try {
                    $provided = $registered->provider->getClassInitializers(
                        new ClassInitializerProviderContext(
                            $this->phpVersion,
                            $codebase,
                            $request->class,
                            $cancellation,
                        ),
                    );
                    foreach ($provided as $initializer) {
                        DefinitionName::assertIdentifier($initializer, 'A class initializer method name');
                        $initializers[$initializer] = $initializer;
                    }
                } catch (Throwable $throwable) {
                    throw new ProtocolException(
                        "Class initializer provider in `{$registered->plugin}` failed: {$throwable->getMessage()}",
                        0,
                        $throwable,
                    );
                }
            }

            return AnalyzerProtocol::writeClassInitializerResponse(array_values($initializers));
        }

        if ($kind === AnalyzerProtocol::ISSUE_FILTER_REQUEST) {
            $request = AnalyzerProtocol::readIssueFilterRequest($reader);
            if ($this->metadataCache === null || $this->metadataCache->generation !== $request->generation) {
                $this->metadataCache = new MetadataCache($request->generation);
            }

            $codebase = new Codebase($host, $requestId, $cancellation, $this->metadataCache);
            $types = new TypeComparator($host, $requestId, $cancellation, $this->metadataCache);
            $removed = [];
            foreach ($request->issues as $issueIndex => $issue) {
                $context = new IssueFilterContext(
                    $this->phpVersion,
                    $codebase,
                    $types,
                    $cancellation,
                    $request->file,
                    $request->contents,
                    $issue,
                );

                foreach ($request->hookIndices as $hookIndex) {
                    $registered = $this->issueFilterHooks[$hookIndex] ?? null;
                    if ($registered === null) {
                        throw new ProtocolException(
                            "Mago requested unregistered analyzer issue-filter hook index {$hookIndex}.",
                        );
                    }

                    if ($context->issue->code === null || !in_array($context->issue->code, $registered->codes, true)) {
                        continue;
                    }

                    $cancellation->throwIfCancelled();
                    try {
                        $decision = $registered->hook->filterIssue($context);
                    } catch (Throwable $throwable) {
                        throw new ProtocolException(
                            "Analyzer issue-filter hook in `{$registered->plugin}` failed: {$throwable->getMessage()}",
                            0,
                            $throwable,
                        );
                    }

                    if ($decision === IssueFilterDecision::Remove) {
                        $removed[] = $issueIndex;
                        break;
                    }
                }
            }

            return AnalyzerProtocol::writeIssueFilterResponse(count($request->issues), $removed);
        }

        if ($kind !== AnalyzerProtocol::RETURN_TYPE_REQUEST && $kind !== AnalyzerProtocol::CALLABLE_SIGNATURE_REQUEST) {
            throw new ProtocolException("Unknown analyzer request kind {$kind}.");
        }

        $request = AnalyzerProtocol::readReturnTypeRequest($reader);
        if ($this->metadataCache === null || $this->metadataCache->generation !== $request->generation) {
            $this->metadataCache = new MetadataCache($request->generation);
        }
        $codebase = new Codebase($host, $requestId, $cancellation, $this->metadataCache);
        $providers = $request->invocation->kind === InvocationKind::Function
            ? $this->functionReturnTypeProviders
            : $this->methodReturnTypeProviders;
        $types = new TypeComparator($host, $requestId, $cancellation, $this->metadataCache);
        if ($kind === AnalyzerProtocol::CALLABLE_SIGNATURE_REQUEST) {
            $context = new CallableSignatureProviderContext(
                $this->phpVersion,
                $codebase,
                $request->invocation,
                $types,
                $cancellation,
            );
            foreach ($request->providerIndices as $providerIndex) {
                $registered = $providers[$providerIndex] ?? null;
                if ($registered === null) {
                    throw new ProtocolException(
                        "Mago requested unregistered analyzer provider index {$providerIndex}.",
                    );
                }

                $cancellation->throwIfCancelled();
                try {
                    if (!$registered->provider instanceof CallableSignatureProvider) {
                        throw new ProtocolException(
                            "Mago requested a callable signature from analyzer provider {$providerIndex}, but it does not advertise that capability.",
                        );
                    }

                    $signature = $registered->provider->getCallableSignature($context);
                    if ($signature !== null) {
                        return AnalyzerProtocol::writeCallableSignatureResponse($signature);
                    }
                } catch (Throwable $throwable) {
                    throw new ProtocolException(
                        "Analyzer provider in `{$registered->plugin}` failed: {$throwable->getMessage()}",
                        0,
                        $throwable,
                    );
                }
            }

            return AnalyzerProtocol::writeCallableSignatureResponse(null);
        }

        $context = new ReturnTypeProviderContext(
            $this->phpVersion,
            $codebase,
            $request->invocation,
            $types,
            $cancellation,
        );
        foreach ($request->providerIndices as $providerIndex) {
            $registered = $providers[$providerIndex] ?? null;
            if ($registered === null) {
                throw new ProtocolException("Mago requested unregistered analyzer provider index {$providerIndex}.");
            }

            $cancellation->throwIfCancelled();
            try {
                $type = $registered->provider->getReturnType($context);
            } catch (Throwable $throwable) {
                throw new ProtocolException(
                    "Analyzer provider in `{$registered->plugin}` failed: {$throwable->getMessage()}",
                    0,
                    $throwable,
                );
            }

            if ($type !== null) {
                return AnalyzerProtocol::writeReturnTypeResponse($type);
            }
        }

        return AnalyzerProtocol::writeReturnTypeResponse(null);
    }

    private function handleAnalyzerInitialization(
        PayloadReader $reader,
        CancellationTokenInterface $cancellation,
    ): string {
        $pluginIndices = AnalyzerProtocol::readInitializationRequest($reader);
        $stubs = [];
        foreach ($pluginIndices as $pluginIndex) {
            $registered = $this->analyzerPlugins[$pluginIndex] ?? null;
            if ($registered === null) {
                throw new ProtocolException("Mago initialized unregistered analyzer plugin index {$pluginIndex}.");
            }

            $cancellation->throwIfCancelled();
            $context = new InitializationContext($this->phpVersion, $cancellation);
            try {
                foreach ($registered->initializationHooks as $hook) {
                    $hook->initialize($context);
                }
            } catch (Throwable $throwable) {
                throw new ProtocolException(
                    "Analyzer plugin `{$registered->definition->identifier}` failed to initialize: {$throwable->getMessage()}",
                    0,
                    $throwable,
                );
            }

            $stubs[$pluginIndex] = $context->getStubs();
        }

        return AnalyzerProtocol::writeInitializationResponse($stubs);
    }

    /**
     * @param positive-int $requestId
     * @mago-expect lint:halstead
     */
    private function handleAnalyzerLifecycleRequest(
        int $kind,
        PayloadReader $reader,
        int $requestId,
        HostClient $host,
        CancellationTokenInterface $cancellation,
    ): string {
        $request = AnalyzerProtocol::readLifecycleRequest(
            $kind,
            $reader,
            $host,
            $requestId,
            $cancellation,
            $this->phpVersion,
            $this->nodeKinds,
        );
        if ($this->metadataCache === null || $this->metadataCache->generation !== $request->generation) {
            $this->metadataCache = new MetadataCache($request->generation);
        }
        $codebase = new Codebase($host, $requestId, $cancellation, $this->metadataCache);
        $types = new TypeComparator($host, $requestId, $cancellation, $this->metadataCache);
        $reportedIssues = [];
        $contributedReferences = [];
        $analyses = is_array($request->analysis) ? $request->analysis : [$request->analysis];
        foreach ($analyses as $analysis) {
            foreach ($request->pluginIndices as $pluginIndex) {
                $registered = $this->analyzerPlugins[$pluginIndex] ?? null;
                if ($registered === null) {
                    throw new ProtocolException("Mago requested unregistered analyzer plugin index {$pluginIndex}.");
                }

                $cancellation->throwIfCancelled();
                try {
                    $context = match ($kind) {
                        AnalyzerProtocol::BEFORE_ANALYSIS_REQUEST => new BeforeAnalysisContext(
                            $this->phpVersion,
                            $codebase,
                            $types,
                            $cancellation,
                        ),
                        AnalyzerProtocol::AFTER_FILE_ANALYSIS_REQUEST => new AfterFileAnalysisContext(
                            $this->phpVersion,
                            $codebase,
                            $types,
                            $cancellation,
                            $analysis instanceof FileAnalysis
                                ? $analysis
                                : throw new ProtocolException('An after-file request has no file analysis.'),
                        ),
                        AnalyzerProtocol::AFTER_FILE_ANALYSIS_BATCH_REQUEST => new AfterFileAnalysisContext(
                            $this->phpVersion,
                            $codebase,
                            $types,
                            $cancellation,
                            $analysis instanceof FileAnalysis
                                ? $analysis
                                : throw new ProtocolException('An after-file batch contains an invalid analysis.'),
                        ),
                        AnalyzerProtocol::AFTER_ANALYSIS_REQUEST => new AfterAnalysisContext(
                            $this->phpVersion,
                            $codebase,
                            $types,
                            $cancellation,
                            $analysis instanceof ProjectAnalysis
                                ? $analysis
                                : throw new ProtocolException('An after-analysis request has no project analysis.'),
                        ),
                        default => throw new ProtocolException("Unknown analyzer lifecycle request kind {$kind}."),
                    };

                    if ($context instanceof BeforeAnalysisContext) {
                        foreach ($registered->beforeAnalysisHooks as $hook) {
                            $hook->beforeAnalysis($context);
                        }
                    }

                    if ($context instanceof AfterFileAnalysisContext) {
                        foreach ($registered->afterFileAnalysisHooks as $hook) {
                            $hook->afterFileAnalysis($context);
                        }

                        $this->runNodeAnalysisHooks($registered, $context, $pluginIndex, $reportedIssues);
                    }

                    if ($context instanceof AfterAnalysisContext) {
                        foreach ($registered->afterAnalysisHooks as $hook) {
                            $hook->afterAnalysis($context);
                        }
                    }
                } catch (Throwable $throwable) {
                    $identifier = $registered->definition->identifier;
                    throw new ProtocolException(
                        "Analyzer lifecycle hook in `{$identifier}` failed: {$throwable->getMessage()}",
                        0,
                        $throwable,
                    );
                }

                foreach ($context->takeReportedIssues() as $issue) {
                    $reportedIssues[] = $pluginIndex;
                    $reportedIssues[] = $issue;
                    $reportedIssues[] = $context instanceof AfterFileAnalysisContext ? $context->analysis->file : null;
                }

                if ($context instanceof BeforeAnalysisContext || $context instanceof AfterFileAnalysisContext) {
                    $references = $context->references->takeReferences();
                    for ($index = 0, $count = count($references); $index < $count; $index += 3) {
                        $contributedReferences[] = $pluginIndex;
                        $contributedReferences[] = $references[$index];
                        $contributedReferences[] = $references[$index + 1];
                        $contributedReferences[] = $references[$index + 2];
                        $contributedReferences[] = $context instanceof AfterFileAnalysisContext
                            ? $context->analysis->file
                            : null;
                    }
                }
            }
        }

        return AnalyzerProtocol::writeLifecycleResponse($kind, $reportedIssues, $contributedReferences);
    }

    /**
     * @param int<0, 65535> $pluginIndex
     * @param list<int|ReportedIssue|string|null> $reportedIssues
     */
    private function runNodeAnalysisHooks(
        RegisteredPlugin $registered,
        AfterFileAnalysisContext $context,
        int $pluginIndex,
        array &$reportedIssues,
    ): void {
        if ($registered->nodeAnalysisHooks === []) {
            return;
        }

        $source = $context->analysis->getSourceFile();
        $targetIndex = 0;
        foreach ($source->getTargetNodes() as $node) {
            if (($targetIndex++ & 63) === 0) {
                $context->cancellation->throwIfCancelled();
            }

            $hooks = $registered->nodeAnalysisHooksByNodeKind[$node->kind->value] ?? [];
            if ($hooks === []) {
                continue;
            }

            $nodeContext = new NodeAnalysisContext($context, $source, $node);
            foreach ($hooks as $hook) {
                $hook->hook->analyze($nodeContext);
                foreach ($nodeContext->takeReportedIssues() as $issue) {
                    $reportedIssues[] = $pluginIndex;
                    $reportedIssues[] = $issue;
                    $reportedIssues[] = $context->analysis->file;
                }
            }
        }
    }

    /**
     * @param array<int<0, max>, SignalCancellationToken> $requests
     */
    private function cancelRequests(array $requests): void
    {
        foreach ($requests as $request) {
            $request->cancel();
        }
    }

    private function captureAccidentalOutput(): int
    {
        $level = ob_get_level();
        ob_start(static function (string $output): string {
            if ($output !== '' && defined('STDERR')) {
                fwrite(STDERR, $output);
            }

            return '';
        }, 1);

        return $level;
    }

    private function writeThrowable(Throwable $throwable): void
    {
        fwrite(STDERR, $throwable::class . ': ' . $throwable->getMessage() . "\n");
    }
}
