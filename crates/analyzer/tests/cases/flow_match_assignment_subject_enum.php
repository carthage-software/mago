<?php

declare(strict_types=1);

enum FlowMatchAssignmentStatus
{
    case Draft;
    case Published;
    case Archived;
}

function flow_match_assignment_current_status(): FlowMatchAssignmentStatus
{
    return FlowMatchAssignmentStatus::Draft;
}

/**
 * @param FlowMatchAssignmentStatus::Archived $status
 */
function flow_match_assignment_take_archived(FlowMatchAssignmentStatus $status): string
{
    return $status->name;
}

function flow_match_assignment_subject_enum(): string
{
    return match ($status = flow_match_assignment_current_status()) {
        FlowMatchAssignmentStatus::Draft => 'draft',
        FlowMatchAssignmentStatus::Published => 'published',
        FlowMatchAssignmentStatus::Archived => 'archived ' . $status->name,
    };
}

function flow_match_assignment_subject_enum_default(): string
{
    return match ($status = flow_match_assignment_current_status()) {
        FlowMatchAssignmentStatus::Draft => 'draft',
        FlowMatchAssignmentStatus::Published => 'published',
        default => flow_match_assignment_take_archived($status),
    };
}

function flow_match_assignment_subject_enum_nested(): string
{
    return match ($outer = $inner = flow_match_assignment_current_status()) {
        FlowMatchAssignmentStatus::Draft => 'draft',
        FlowMatchAssignmentStatus::Published => 'published',
        FlowMatchAssignmentStatus::Archived => $outer->name . $inner->name,
    };
}

function flow_match_assignment_subject_enum_parenthesized(): string
{
    return match (($status = flow_match_assignment_current_status())) {
        FlowMatchAssignmentStatus::Draft => 'draft',
        FlowMatchAssignmentStatus::Published => 'published',
        FlowMatchAssignmentStatus::Archived => flow_match_assignment_take_archived($status),
    };
}

function flow_match_assignment_subject_enum_coalesce(?FlowMatchAssignmentStatus $status): string
{
    return match ($status ??= flow_match_assignment_current_status()) {
        FlowMatchAssignmentStatus::Draft => 'draft',
        FlowMatchAssignmentStatus::Published => 'published',
        FlowMatchAssignmentStatus::Archived => flow_match_assignment_take_archived($status),
    };
}
