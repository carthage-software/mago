<?php

declare(strict_types=1);

enum FlowMatchAssignmentTargetStatus
{
    case Draft;
    case Published;
    case Archived;
}

function flow_match_assignment_target_status(): FlowMatchAssignmentTargetStatus
{
    return FlowMatchAssignmentTargetStatus::Draft;
}

/**
 * @param FlowMatchAssignmentTargetStatus::Archived $status
 */
function flow_match_assignment_target_take_archived(FlowMatchAssignmentTargetStatus $status): string
{
    return $status->name;
}

final class FlowMatchAssignmentTargetNode
{
    public function value(): int
    {
        return 1;
    }
}

final class FlowMatchAssignmentTarget
{
    public FlowMatchAssignmentTargetStatus $status = FlowMatchAssignmentTargetStatus::Draft;
    public ?FlowMatchAssignmentTargetNode $current = null;
    public int $count = 0;
    public static ?FlowMatchAssignmentTargetNode $shared = null;

    public function next(): ?FlowMatchAssignmentTargetNode
    {
        return null;
    }

    public function property(): string
    {
        return match ($this->status = flow_match_assignment_target_status()) {
            FlowMatchAssignmentTargetStatus::Draft => 'draft',
            FlowMatchAssignmentTargetStatus::Published => 'published',
            FlowMatchAssignmentTargetStatus::Archived => flow_match_assignment_target_take_archived($this->status),
        };
    }

    public function propertyDefault(): string
    {
        return match ($this->status = flow_match_assignment_target_status()) {
            FlowMatchAssignmentTargetStatus::Draft => 'draft',
            FlowMatchAssignmentTargetStatus::Published => 'published',
            default => flow_match_assignment_target_take_archived($this->status),
        };
    }

    public function nullableProperty(): int
    {
        return match ($this->current = $this->next()) {
            null => 0,
            default => $this->current->value(),
        };
    }

    public function staticProperty(?FlowMatchAssignmentTargetNode $node): int
    {
        return match (self::$shared = $node) {
            null => 0,
            default => self::$shared->value(),
        };
    }

    public function literalProperty(): string
    {
        return match ($this->count = 3) {
            3 => 'three',
        };
    }

    /**
     * @mago-expect analysis:match-arm-always-true(1)
     * @mago-expect analysis:unreachable-match-default-arm(1)
     */
    public function literalPropertyDefault(): string
    {
        return match ($this->count = 3) {
            3 => 'three',
            default => 'other',
        };
    }
}

/**
 * @param array<string, ?FlowMatchAssignmentTargetNode> $nodes
 */
function flow_match_assignment_subject_target_narrowing(array $nodes, ?FlowMatchAssignmentTargetNode $node): int
{
    return match ($nodes['key'] = $node) {
        null => 0,
        default => $nodes['key']->value(),
    };
}

final class FlowMatchAssignmentTargetHooked
{
    public FlowMatchAssignmentTargetStatus $status = FlowMatchAssignmentTargetStatus::Draft {
        set(FlowMatchAssignmentTargetStatus|string $value) {
            $this->status = $value instanceof FlowMatchAssignmentTargetStatus
                ? $value
                : FlowMatchAssignmentTargetStatus::Draft;
        }
    }

    public function hooked(): string
    {
        return match ($this->status = flow_match_assignment_target_status()) {
            FlowMatchAssignmentTargetStatus::Draft => 'draft',
            FlowMatchAssignmentTargetStatus::Published => 'published',
            FlowMatchAssignmentTargetStatus::Archived => flow_match_assignment_target_take_archived($this->status),
        };
    }
}

/**
 * @property-read FlowMatchAssignmentTargetStatus $status
 * @property-write FlowMatchAssignmentTargetStatus|string $status
 */
final class FlowMatchAssignmentTargetMagic
{
    /** @var array<string, mixed> */
    private array $data = [];

    public function __get(string $name): mixed
    {
        return $this->data[$name] ?? null;
    }

    public function __set(string $name, mixed $value): void
    {
        $this->data[$name] = $value;
    }

    public function magic(): string
    {
        return match ($this->status = flow_match_assignment_target_status()) {
            FlowMatchAssignmentTargetStatus::Draft => 'draft',
            FlowMatchAssignmentTargetStatus::Published => 'published',
            default => flow_match_assignment_target_take_archived($this->status),
        };
    }
}

final class FlowMatchAssignmentTargetOuter
{
    public ?FlowMatchAssignmentTarget $target = null;
}

function flow_match_assignment_subject_target_narrowing_nested(
    FlowMatchAssignmentTargetOuter $outer,
    ?FlowMatchAssignmentTargetNode $node,
): int {
    if ($outer->target === null) {
        return 0;
    }

    return match ($outer->target->current = $node) {
        null => 0,
        default => $outer->target->current->value(),
    };
}
