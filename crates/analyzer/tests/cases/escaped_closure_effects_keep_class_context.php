<?php
class Base { public function baseMethod(): void {} }
class Child extends Base {}
/**
 * @template T
 * @param Closure(): T $factory
 * @param-closure-this Child $factory
 * @return T
 */
function in_child(Closure $factory): mixed { return $factory(); }
function require_base(Base $value): void {}
$write = in_child(function () {
    return /** @param-out parent $value */
        static function (&$value): void { $value = new Base(); };
});
$value = null;
$write($value);
require_base($value);
$partial = $write(value: ?);
$value = null;
$partial($value);
require_base($value);
$first = $write(...);
$value = null;
$first($value);
require_base($value);

$bind = in_child(function () {
    return /** @param-closure-this parent $callback */
        static function (Closure $callback): void {};
});
$bind(function (): void { $this->baseMethod(); });
$bindPartial = $bind(callback: ?);
$bindPartial(function (): void { $this->baseMethod(); });
