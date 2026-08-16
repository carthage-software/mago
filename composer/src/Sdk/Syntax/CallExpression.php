<?php

declare(strict_types=1);

namespace Mago\Sdk\Syntax;

use Mago\Sdk\Exception\InvalidArgumentException;

use function str_starts_with;

/**
 * A structured view over a function, method, or static-method call.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 */
final class CallExpression
{
    /** @var list<CallArgument> */
    public readonly array $arguments;

    /**
     * @param list<CallArgument> $arguments
     */
    private function __construct(
        public readonly Node $node,
        public readonly Node $callee,
        public readonly ?Node $receiver,
        public readonly ?Node $member,
        array $arguments,
    ) {
        $this->arguments = $arguments;
    }

    public static function fromNode(SourceFile $source, Node $node): self
    {
        if (
            $node->kind !== NodeKind::FunctionCall
            && $node->kind !== NodeKind::MethodCall
            && $node->kind !== NodeKind::NullSafeMethodCall
            && $node->kind !== NodeKind::StaticMethodCall
        ) {
            throw new InvalidArgumentException('A call-expression view requires a call node.');
        }

        $children = $source->getChildren($node);
        $function = $node->kind === NodeKind::FunctionCall;
        $callee = self::unwrapExpression(
            $source,
            $children[0] ?? throw new InvalidArgumentException('A call node has no callee.'),
        );
        $member = $function ? null : $children[1] ?? null;
        $argumentList = $children[$function ? 1 : 2] ?? null;
        if ($argumentList?->kind !== NodeKind::ArgumentList) {
            throw new InvalidArgumentException('A call node has no argument list.');
        }

        $arguments = [];
        foreach ($source->getChildren($argumentList) as $argument) {
            $variant = $source->getChildren($argument)[0] ?? null;
            if ($variant === null) {
                continue;
            }
            $parts = $source->getChildren($variant);
            $named = $variant->kind === NodeKind::NamedArgument;
            $value = $parts[$named ? 1 : 0] ?? null;
            if ($value === null) {
                continue;
            }
            $value = self::unwrapExpression($source, $value);
            $arguments[] = new CallArgument(
                $argument,
                $value,
                $named ? $source->getText($parts[0]) : null,
                str_starts_with($source->getText($variant), '...'),
            );
        }

        return new self($node, $callee, $function ? null : $callee, $member, $arguments);
    }

    public static function fromExpression(SourceFile $source, Node $node): ?self
    {
        $node = self::unwrapExpression($source, $node);
        while ($node->kind === NodeKind::Call) {
            $next = $source->getChildren($node)[0] ?? null;
            if ($next === null) {
                break;
            }
            $node = self::unwrapExpression($source, $next);
        }

        return match ($node->kind) {
            NodeKind::FunctionCall,
            NodeKind::MethodCall,
            NodeKind::NullSafeMethodCall,
            NodeKind::StaticMethodCall,
                => self::fromNode($source, $node),
            default => null,
        };
    }

    public function isFunction(): bool
    {
        return $this->node->kind === NodeKind::FunctionCall;
    }

    public function isStaticMethod(): bool
    {
        return $this->node->kind === NodeKind::StaticMethodCall;
    }

    public function isMethod(): bool
    {
        return $this->node->kind === NodeKind::MethodCall || $this->node->kind === NodeKind::NullSafeMethodCall;
    }

    public function getName(SourceFile $source): ?string
    {
        if ($this->isFunction()) {
            $callee = $this->callee;
            while ($callee->kind === NodeKind::ConstantAccess || $callee->kind === NodeKind::Expression) {
                $next = $source->getChildren($callee)[0] ?? null;
                if ($next === null) {
                    break;
                }
                $callee = $next;
            }

            return match ($callee->kind) {
                NodeKind::Identifier,
                NodeKind::LocalIdentifier,
                NodeKind::QualifiedIdentifier,
                NodeKind::FullyQualifiedIdentifier,
                    => $source->getText($callee),
                default => null,
            };
        }

        if ($this->member === null) {
            return null;
        }
        $selector = $source->getChildren($this->member)[0] ?? null;

        return $selector?->kind === NodeKind::LocalIdentifier ? $source->getText($selector) : null;
    }

    private static function unwrapExpression(SourceFile $source, Node $node): Node
    {
        while ($node->kind === NodeKind::Expression) {
            $next = $source->getChildren($node)[0] ?? null;
            if ($next === null) {
                break;
            }
            $node = $next;
        }

        return $node;
    }
}
