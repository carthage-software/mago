<?php

declare(strict_types=1);

namespace App;

use Attribute;

#[Attribute(Attribute::TARGET_CLASS)]
final class ParameterAttribute
{
    public function __construct(
        public string $name,
        public mixed $schema = null,
    ) {}
}

class Schema
{
    public function __construct(
        mixed $field00 = null,
        mixed $field01 = null,
        mixed $field02 = null,
        mixed $field03 = null,
        mixed $field04 = null,
        mixed $field05 = null,
        mixed $field06 = null,
        mixed $field07 = null,
        mixed $field08 = null,
        mixed $field09 = null,
        mixed $field10 = null,
        mixed $field11 = null,
        mixed $field12 = null,
        mixed $field13 = null,
        mixed $field14 = null,
        mixed $field15 = null,
        mixed $field16 = null,
        mixed $field17 = null,
        mixed $field18 = null,
        mixed $field19 = null,
        mixed $field20 = null,
        mixed $field21 = null,
        mixed $field22 = null,
        mixed $field23 = null,
        mixed $field24 = null,
        mixed $field25 = null,
        mixed $field26 = null,
        mixed $field27 = null,
        mixed $field28 = null,
        mixed $field29 = null,
        mixed $field30 = null,
        mixed $field31 = null,
        mixed $field32 = null,
        mixed $field33 = null,
        mixed $field34 = null,
        mixed $field35 = null,
        mixed $field36 = null,
        mixed $field37 = null,
        mixed $field38 = null,
        mixed $field39 = null,
        mixed $field40 = null,
        mixed $field41 = null,
        mixed $field42 = null,
        mixed $field43 = null,
        mixed $field44 = null,
        mixed $field45 = null,
        mixed $field46 = null,
        mixed $field47 = null,
        mixed $field48 = null,
        mixed $field49 = null,
        mixed $field50 = null,
        mixed $field51 = null,
        int $field52 = 0,
        mixed $field53 = null,
        mixed $field54 = null,
        mixed $field55 = null,
        string $field56 = '',
        string $field57 = '',
        mixed $field58 = null,
    ) {}
}

final class Items extends Schema {}

#[ParameterAttribute(
    name: 'page',
    schema: new Schema(
        field58: new Items(field57: 'string', field56: 'enum'),
        field55: 'array',
        field54: 'example',
        field53: ['draft'],
    ),
)]
final class NamedArgumentReproduction {}

#[ParameterAttribute('page', new Schema(new Items('string', 'enum'), 'array', 'example', ['draft']))]
final class PositionalArgumentReproduction {}

new Schema(field52: 52, field57: 'last', field56: 'before');
new Schema(field52: 'wrong'); // @mago-expect analysis:invalid-argument
new Schema(...['field57' => 'last', 'field56' => 'before']);
new Schema(
    field58: '58',
    field57: '57',
    field56: '56',
    field55: '55',
    field54: '54',
    field53: '53',
    field52: 52,
    field51: '51',
    field50: '50',
    field49: '49',
    field48: '48',
    field47: '47',
    field46: '46',
    field45: '45',
    field44: '44',
    field43: '43',
    field42: '42',
    field41: '41',
    field40: '40',
    field39: '39',
    field38: '38',
    field37: '37',
    field36: '36',
    field35: '35',
    field34: '34',
    field33: '33',
    field32: '32',
    field31: '31',
    field30: '30',
    field29: '29',
    field28: '28',
    field27: '27',
    field26: '26',
    field25: '25',
    field24: '24',
    field23: '23',
    field22: '22',
    field21: '21',
    field20: '20',
    field19: '19',
    field18: '18',
    field17: '17',
    field16: '16',
    field15: '15',
    field14: '14',
    field13: '13',
    field12: '12',
    field11: '11',
    field10: '10',
    field09: '9',
    field08: '8',
    field07: '7',
    field06: '6',
    field05: '5',
    field04: '4',
    field03: '3',
    field02: '2',
    field01: '1',
    field00: '0',
);

/** @mago-expect analysis:invalid-argument,duplicate-named-argument,invalid-named-argument */
new Schema(
    field00: null,
    field01: null,
    field02: null,
    field03: null,
    field04: null,
    field05: null,
    field06: null,
    field07: null,
    field08: null,
    field09: null,
    field10: null,
    field11: null,
    field12: null,
    field13: null,
    field14: null,
    field15: null,
    field16: null,
    field17: null,
    field18: null,
    field19: null,
    field20: null,
    field21: null,
    field22: null,
    field23: null,
    field24: null,
    field25: null,
    field26: null,
    field27: null,
    field28: null,
    field29: null,
    field52: 'wrong',
    field52: 52,
    Field52: 52,
);
