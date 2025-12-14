<?php

declare(strict_types=1);

/**
 * @require-extends AbstractTestCase
 */
trait LocaleAwareTestCaseTrait
{
    public function tearDown(): void
    {
        parent::tearDown();
    }
}

abstract class AbstractTestCase
{
    public function tearDown(): void
    {
    }
}

class MyTest extends AbstractTestCase
{
    use LocaleAwareTestCaseTrait;
}
