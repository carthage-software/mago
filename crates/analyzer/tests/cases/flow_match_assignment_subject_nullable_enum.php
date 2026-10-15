<?php

declare(strict_types=1);

enum FlowMatchAssignmentNullableStatus
{
    case Draft;
    case Published;
    case Archived;
}

function flow_match_assignment_nullable_status(): ?FlowMatchAssignmentNullableStatus
{
    return null;
}

function flow_match_assignment_subject_nullable_enum(): string
{
    return match ($status = flow_match_assignment_nullable_status()) {
        null => 'none',
        FlowMatchAssignmentNullableStatus::Draft => 'draft',
        FlowMatchAssignmentNullableStatus::Published => 'published',
        FlowMatchAssignmentNullableStatus::Archived => $status->name,
    };
}
