<?php

namespace App;

/** @template Item */
class Generic {}

/** @return Item */
function sibling() {}

namespace Other;

/** @return Item */
function control() {}
