<?php

/** @require-extends AbstractTestCase */
trait LocaleAwareTestCaseTrait
{
    private const string DEFAULT_LOCALE = 'de';
    protected static string $currentLocale = self::DEFAULT_LOCALE;

    protected function tearDown(): void
    {
        parent::tearDown();
        static::$currentLocale = self::DEFAULT_LOCALE;
    }
}

abstract class AbstractTestCase
{
    protected function tearDown(): void {}
}

class MyTest extends AbstractTestCase
{
    use LocaleAwareTestCaseTrait;
}
