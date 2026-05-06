<?php

function testFinallyAfterReturn(): void
{
    try {
        return;
    } finally {
        echo 'finally runs';
    }
}

/**
 * @throws Exception
 */
function testFinallyAfterThrow(): void
{
    try {
        throw new Exception();
    } finally {
        echo 'finally runs';
    }
}
