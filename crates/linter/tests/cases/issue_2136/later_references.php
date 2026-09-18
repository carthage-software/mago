<?php

namespace App;

function imported(
    \Other\Base $base,
    \Other\Contract $contract,
    \Other\Behavior $behavior,
    \Other\Marker $marker,
    \Other\Hinted $hinted,
    \Other\Created $created,
    \Other\Checked $checked,
    \Other\Called $called,
    \Other\Callback $callback,
    \Other\Stored $stored,
    \Other\ConstantOwner $constantOwner,
    \Other\Caught $caught,
    \Other\ClassPrefix $classPrefix,
    \Other\FunctionPrefix $functionPrefix,
    \Other\CallbackPrefix $callbackPrefix,
    \Other\ConstantPrefix $constantPrefix,
    \Other\LocalClass $localClass,
    \Other\LocalInterface $localInterface,
    \Other\LocalTrait $localTrait,
    \Other\LocalEnum $localEnum,
): void {}

#[Marker]
class Example extends Base implements Contract
{
    use Behavior;

    public function run(Hinted $value): void
    {
        new created();
        $value instanceof Checked;
        Called::run();
        Callback::run(...);
        Stored::$value;
        ConstantOwner::VALUE;
        try {} catch (Caught) {}
        new ClassPrefix\Value();
        FunctionPrefix\run();
        CallbackPrefix\run(...);
        ConstantPrefix\VALUE;
    }
}

class LocalClass {}
interface LocalInterface {}
trait LocalTrait {}
enum LocalEnum {}
