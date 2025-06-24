<?php

const ZEND_ACC_PUBLIC = 1;

const ZEND_ACC_PROTECTED = 2;

const ZEND_ACC_PRIVATE = 4;

const ZEND_ACC_STATIC = 16;

const ZEND_ACC_FINAL = 32;

const ZEND_ACC_ABSTRACT = 64;

const ZEND_ACC_FETCH = PHP_INT_MAX;

const ZEND_ACC_PPP_MASK = ZEND_ACC_PUBLIC | ZEND_ACC_PROTECTED | ZEND_ACC_PRIVATE;

function uopz_add_function(...$arguments): bool
{
}

function uopz_allow_exit(...$arguments): void
{
}

function uopz_del_function(...$arguments): bool
{
}

function uopz_extend(...$arguments): bool
{
}

function uopz_flags(...$arguments): int
{
}

function uopz_get_exit_status(): null|int
{
}

function uopz_get_hook(...$arguments): null|Closure
{
}

function uopz_get_mock(...$arguments)
{
}

function uopz_get_property(...$arguments)
{
}

function uopz_get_return(...$arguments)
{
}

function uopz_get_static(...$arguments): null|array
{
}

function uopz_implement(...$arguments): bool
{
}

function uopz_redefine(...$arguments): bool
{
}

function uopz_set_hook(...$arguments): bool
{
}

function uopz_set_mock(...$arguments): void
{
}

function uopz_set_property(...$arguments): void
{
}

function uopz_set_return(...$arguments): bool
{
}

function uopz_set_static(...$arguments): void
{
}

function uopz_undefine(...$arguments): bool
{
}

function uopz_unset_hook(...$arguments): bool
{
}

function uopz_unset_mock(...$arguments): void
{
}

function uopz_unset_return(...$arguments): bool
{
}

function uopz_call_user_func(...$arguments)
{
}

function uopz_call_user_func_array(...$arguments)
{
}
