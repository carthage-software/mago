<?php

function x(): iterable
{
    yield 'INSERT IGNORE INTO `table1` (`col1`, `col2`, `col3`)'
        . ' SELECT `col1`, `col2`, `col3` FROM table2' // comment 1
        . ' WHERE `col4` = "val1"'; // comment 2
}
