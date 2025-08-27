<?php

if (
    aLongishFunction() !== 'a longish value'
    || (in_array($another_val, getListOfValsToCheckAgainst())
    || ($another_val == 1 && !is_foo()))
) {
}