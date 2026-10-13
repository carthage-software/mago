<?php

use AllowDynamicProperties as DynamicProperties;

class Base {}

class Child extends Base {}

class Leaf extends Child {}

#[DynamicProperties]
class Own extends Base {}

class Unrelated {}
