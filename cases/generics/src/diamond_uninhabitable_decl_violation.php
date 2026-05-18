<?php

declare(strict_types=1);

namespace Generics\Test\DiamondUninhabitableDecl;

use Generics\Producer;

/** @extends Producer<int> */
interface IntProducer extends Producer {}

/** @extends Producer<string> */
interface StringProducer extends Producer {}

interface FlexProducer extends IntProducer, StringProducer {}
