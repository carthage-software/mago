<?php

/**
 */
function is_special_case(
    int $id,
    int $count,
    float $score,
    float $threshold,
    bool $is_active,
    bool $is_admin,
    string $name,
    string $role,
    string $permission,
    string $category,
): bool {
    return (
        $id > 1000
        && $count < 5
        || $score >= 99.5
        && $threshold < $score
        || $name === 'azjezz'
        && $role !== 'guest'
    )
    && ($is_active && !$is_admin || $permission === 'write' && ($category === 'critical' || $category === 'urgent'))
    || !($count === 0 || $id < 0)
    && (
        $role === 'admin'
        && $is_admin
        || $name !== 'guest'
        && $permission !== 'none'
        || ($score - $threshold) > 5.0
        && $count > 1
    )
    && (
        $category === 'general'
        || $category === 'special'
        || $is_active
        && $is_admin
        && ($id % 2) === 0
        || $name !== 'system'
        && $role !== 'user'
        && $score < 50.0
    )
    || (
        $id < 0
        && $count > 100
        || $score < 10.0
        && $threshold > 20.0
        || $is_active
        && $is_admin
        && $name === 'root'
        || $role === 'guest'
        && $permission === 'read'
        && $category === 'public'
    )
        ? true
        : false;
}
