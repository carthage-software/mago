<?php

declare(strict_types=1);

interface Ok {}

// @mago-expect analysis:invalid-extend
interface I extends Ok, Ok {}

// @mago-expect analysis:invalid-implement
class C implements Ok, Ok {}

// @mago-expect analysis:invalid-implement
enum E implements Ok, Ok {}

// @mago-expect analysis:invalid-extend
interface A extends B {}

// @mago-expect analysis:invalid-extend
interface B extends A {}
