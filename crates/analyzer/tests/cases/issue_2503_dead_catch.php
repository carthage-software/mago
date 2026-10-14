<?php

declare(strict_types=1);

class Issue2503BaseException extends \RuntimeException {}

class Issue2503ChildException extends Issue2503BaseException {}

class Issue2503OtherException extends \LogicException {}

class Issue2503ThrowingException extends \RuntimeException
{
    /**
     * @mago-expect analysis:unhandled-thrown-type
     */
    public function __construct()
    {
        throw new \LogicException();
    }
}

trait Issue2503ThrowingConstructorAliasTrait
{
    public function create(): void
    {
        // @mago-expect analysis:unhandled-thrown-type
        throw new \LogicException();
    }
}

class Issue2503AliasedConstructorException extends \RuntimeException
{
    use Issue2503ThrowingConstructorAliasTrait {
        create as __construct;
    }
}

/** @mago-expect analysis:dead-catch */
function issue_2503_dead_catch_on_empty_try(): void
{
    try {
    } catch (\RuntimeException) {
    }
}

/** @mago-expect analysis:dead-catch */
function issue_2503_dead_catch_after_parent(): void
{
    try {
        throw new Issue2503ChildException();
    } catch (Issue2503BaseException) {
    } catch (Issue2503ChildException) {
    }
}

function issue_2503_union_catch_can_match_child(): void
{
    try {
        throw new Issue2503ChildException();
    } catch (Issue2503OtherException | Issue2503BaseException) {
    }
}

/**
 * @throws Issue2503BaseException
 */
function issue_2503_throw_base_type(): void
{
    throw new Issue2503ChildException();
}

/**
 * @throws \LogicException
 */
function issue_2503_throw_logic_exception(): void {}

/**
 * @throws Issue2503BaseException
 */
function issue_2503_child_catch_can_match_declared_parent(): void
{
    try {
        issue_2503_throw_base_type();
    } catch (Issue2503ChildException) {
    }
}

/** @mago-expect analysis:dead-catch */
function issue_2503_declared_unrelated_throw_is_dead_catch(): void
{
    try {
        issue_2503_throw_logic_exception();
    } catch (\RuntimeException) {
    } catch (\LogicException) {
    }
}

/** @mago-expect analysis:dead-catch */
function issue_2503_nested_documented_throw_is_handled_locally(): void
{
    try {
        try {
            issue_2503_throw_logic_exception();
        } catch (\LogicException) {
        }
    } catch (\RuntimeException) {
    }
}

/**
 * @throws Issue2503BaseException
 * @mago-expect analysis:dead-catch
 */
function issue_2503_previous_subtype_catch_consumes_declared_parent(): void
{
    try {
        issue_2503_throw_base_type();
    } catch (Issue2503ChildException) {
    } catch (Issue2503ChildException) {
    }
}

/** @mago-expect analysis:dead-catch */
function issue_2503_nested_exception_is_handled_locally(): void
{
    try {
        try {
            throw new Issue2503ChildException();
        } catch (Issue2503ChildException) {
        }
    } catch (Issue2503ChildException) {
    }
}

function issue_2503_nested_exception_is_propagated(): void
{
    try {
        try {
            throw new Issue2503ChildException();
        } catch (Issue2503ChildException $exception) {
            throw $exception;
        }
    } catch (Issue2503BaseException) {
    }
}

/**
 * @throws \RuntimeException
 * @mago-expect analysis:dead-catch(2)
 */
function issue_2503_catch_body_does_not_reach_sibling_catch(): void
{
    try {
    } catch (\InvalidArgumentException) {
        throw new \RuntimeException();
    } catch (\RuntimeException) {
    }
}

function issue_2503_catches_undocumented_exception_constructor_throw(): void
{
    try {
        new Issue2503ThrowingException();
    } catch (\LogicException) {
    }
}

/** @mago-expect analysis:dead-catch */
function issue_2503_builtin_constructor_does_not_hide_dead_catch(): void
{
    try {
        new \RuntimeException();
    } catch (\LogicException) {
    }
}

function issue_2503_catches_aliased_trait_constructor_throw(): void
{
    try {
        new Issue2503AliasedConstructorException();
    } catch (\LogicException) {
    }
}

/**
 * @throws Issue2503BaseException
 * @throws \RuntimeException
 */
function issue_2503_run_cases(): void
{
    issue_2503_dead_catch_on_empty_try();
    issue_2503_dead_catch_after_parent();
    issue_2503_union_catch_can_match_child();
    issue_2503_child_catch_can_match_declared_parent();
    issue_2503_previous_subtype_catch_consumes_declared_parent();
    issue_2503_nested_exception_is_handled_locally();
    issue_2503_nested_exception_is_propagated();
    issue_2503_catch_body_does_not_reach_sibling_catch();
    issue_2503_declared_unrelated_throw_is_dead_catch();
    issue_2503_nested_documented_throw_is_handled_locally();
    issue_2503_catches_undocumented_exception_constructor_throw();
    issue_2503_builtin_constructor_does_not_hide_dead_catch();
    issue_2503_catches_aliased_trait_constructor_throw();
}

issue_2503_run_cases();
