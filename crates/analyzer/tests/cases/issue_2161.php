<?php

declare(strict_types=1);

#[\Attribute(\Attribute::TARGET_ALL | \Attribute::IS_REPEATABLE)]
final class TypedAttribute
{
    public function __construct(
        public string $value,
        public int $count,
    ) {}
}

// @mago-expect analysis:invalid-argument
#[TypedAttribute(2, 1)]
class AttributeWithInvalidArgument {}

// @mago-expect analysis:too-few-arguments
#[TypedAttribute('value')]
class AttributeWithTooFewArguments {}

// @mago-expect analysis:too-few-arguments
#[TypedAttribute]
class AttributeWithNoArguments {}

// @mago-expect analysis:too-many-arguments
#[TypedAttribute('value', 1, 2)]
class AttributeWithTooManyArguments {}

// @mago-expect analysis:invalid-argument
#[TypedAttribute(value: 'value', count: 'one')]
class AttributeWithInvalidNamedArgumentType {}

// @mago-expect analysis:invalid-named-argument
#[TypedAttribute(value: 'value', unknown: 1)]
class AttributeWithUnknownNamedArgument {}

// @mago-expect analysis:invalid-argument
#[TypedAttribute(strtolower([]), 1)]
class AttributeWithInvalidNestedExpression {}

#[\Attribute(\Attribute::TARGET_ALL)]
final class MarkerAttribute {}

// @mago-expect analysis:too-many-arguments
#[MarkerAttribute(1)]
class ConstructorlessAttributeWithArguments {}

#[\Attribute(\Attribute::TARGET_ALL)]
final class PrivateConstructorAttribute
{
    private function __construct() {}
}

// @mago-expect analysis:invalid-method-access
#[PrivateConstructorAttribute]
class AttributeWithInaccessibleConstructor {}

// @mago-expect analysis:invalid-argument
#[TypedAttribute(2, 1)]
function functionWithInvalidAttributeArgument(): void {}

function parameterWithInvalidAttributeArgument(
    // @mago-expect analysis:invalid-argument
    #[TypedAttribute(2, 1)] int $value,
): int {
    return $value;
}
