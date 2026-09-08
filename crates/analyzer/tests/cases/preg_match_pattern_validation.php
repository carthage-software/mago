<?php

declare(strict_types=1);

namespace PregMatchPatternValidation {
    use function preg_match as matchPattern;
    use function preg_match_all as matchAllPatterns;

    function takesFalse(false $_): void {}

    /** @param 1 $_ */
    function takesOne(int $_): void {}

    /** @param 0|false $_ */
    function takesZeroOrFalse(int|false $_): void {}

    /** @param positive-int $_ */
    function takesPositiveInteger(int $_): void {}

    function compareMatches(string $subject, string $pattern): void
    {
        if (\preg_match('/foo/', $subject) > 0) {
        }

        if (matchPattern(subject: $subject, pattern: '~foo~i') > 0) {
        }

        if (matchAllPatterns(subject: $subject, pattern: '/foo/') > 0) {
        }

        if (0 < \preg_match_all($pattern, $subject)) {
        }

        if (\preg_match($pattern, $subject) > 0) {
        }
    }

    function rejectMissingDelimiters(string $subject): void
    {
        // @mago-expect analysis:invalid-argument
        if (\preg_match('foo', $subject) > 0) {
        }

        // @mago-expect analysis:invalid-argument
        if (matchPattern(subject: $subject, pattern: '/foo') > 0) {
        }

        // @mago-expect analysis:invalid-argument
        if (matchAllPatterns(subject: $subject, pattern: '') > 0) {
        }
    }

    function preserveRuntimeFailures(string $subject, int $offset): void
    {
        $matches = [];
        $result = \preg_match('/foo/', $subject, $matches, 0, $offset);
        if ($result === false) {
            takesFalse($result);
        }

        $invalidUtf8Result = \preg_match('/./u', "\xff");
        if ($invalidUtf8Result === false) {
            takesFalse($invalidUtf8Result);
        }

        $allResult = \preg_match_all('/./u', "\xff");
        if ($allResult === false) {
            takesFalse($allResult);
        }
    }

    function narrowResultBranches(string $subject): void
    {
        $result = \preg_match('/foo/', $subject);
        if ($result > 0) {
            takesOne($result);
        } else {
            takesZeroOrFalse($result);
        }

        $allResult = \preg_match_all('/foo/', $subject);
        if (0 < $allResult) {
            takesPositiveInteger($allResult);
        } else {
            takesZeroOrFalse($allResult);
        }
    }
}

namespace PregMatchPatternValidation\Shadowed {
    function preg_match(string $pattern, string $subject): int|false
    {
        return \preg_match($subject, $pattern);
    }

    function compareCustomFunction(string $subject): bool
    {
        return preg_match('not a regular expression', $subject) > 0;
    }
}
