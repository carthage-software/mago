<?php

declare(strict_types=1);

namespace Mago\Sdk;

use Mago\Sdk\Exception\CancelledException;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Io\ResourceReader;
use Mago\Sdk\Internal\Io\ResourceWriter;
use Mago\Sdk\Internal\Linter\Protocol as LinterProtocol;
use Mago\Sdk\Internal\Linter\RegisteredRule;
use Mago\Sdk\Internal\Protocol\Frame;
use Mago\Sdk\Internal\Protocol\FrameCodec;
use Mago\Sdk\Internal\Protocol\FrameKind;
use Mago\Sdk\Internal\SignalCancellationToken;
use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Syntax\NodeKind;
use Revolt\EventLoop;
use Throwable;

use function array_key_exists;
use function array_values;
use function count;
use function defined;
use function fwrite;
use function ob_end_flush;
use function ob_get_level;
use function ob_start;

use const STDERR;
use const STDIN;
use const STDOUT;

/**
 * Persistent worker process serving one or more Mago extensions.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
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

    public function __construct(Extension $extension, Extension ...$additionalExtensions)
    {
        $extensions = [$extension, ...array_values($additionalExtensions)];
        $extensionIdentifiers = [];
        $ruleCodes = [];
        $rules = [];
        foreach ($extensions as $registeredExtension) {
            if (array_key_exists($registeredExtension->identifier, $extensionIdentifiers)) {
                throw new InvalidArgumentException(
                    "Extension `{$registeredExtension->identifier}` is registered more than once.",
                );
            }

            $extensionIdentifiers[$registeredExtension->identifier] = true;
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
        }

        $this->extensions = $extensions;
        $this->rules = $rules;
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
        $reader = new ResourceReader($input ?? STDIN);
        $writer = new ResourceWriter($output ?? STDOUT);

        /** @var array<int<0, max>, SignalCancellationToken> $requests */
        $requests = [];
        $outputBufferLevel = $this->captureAccidentalOutput();
        try {
            while (true) {
                $frame = $codec->read($reader);
                if ($frame === null) {
                    $this->cancelRequests($requests);
                    return;
                }

                if ($frame->kind === FrameKind::Shutdown) {
                    $this->cancelRequests($requests);
                    return;
                }

                if ($frame->kind === FrameKind::Cancel) {
                    if (array_key_exists($frame->id, $requests)) {
                        $requests[$frame->id]->cancel();
                    }

                    continue;
                }

                if ($frame->kind !== FrameKind::Request) {
                    throw new ProtocolException("Unexpected {$frame->kind->name} frame from Mago.");
                }

                if ($frame->flags !== 0 || $frame->parentId !== 0) {
                    throw new ProtocolException('A top-level Mago request contains invalid frame metadata.');
                }

                if (array_key_exists($frame->id, $requests)) {
                    throw new ProtocolException("Mago reused in-flight request identifier {$frame->id}.");
                }

                $cancellation = new SignalCancellationToken();
                $requests[$frame->id] = $cancellation;
                EventLoop::queue(function () use ($frame, $cancellation, $codec, $writer, &$requests): void {
                    try {
                        $payload = $this->handleRequest($frame->payload, $cancellation);
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
            while (ob_get_level() > $outputBufferLevel) {
                ob_end_flush();
            }
        }
    }

    private function handleRequest(string $payload, CancellationTokenInterface $cancellation): string
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
