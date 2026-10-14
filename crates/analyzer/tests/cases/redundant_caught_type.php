<?php

declare(strict_types=1);

class LookupFailed extends \RuntimeException
{
}

final class EntryMissing extends LookupFailed
{
}

/**
 * @mago-expect analysis:redundant-caught-type
 */
function sibling_covers_alternative(): string
{
    try {
        throw new EntryMissing();
    } catch (EntryMissing|\Throwable) {
        return 'failed';
    }
}

/**
 * @mago-expect analysis:redundant-caught-type
 */
function later_sibling_covers_alternative(): string
{
    try {
        throw new EntryMissing();
    } catch (\RuntimeException|EntryMissing) {
        return 'failed';
    }
}

/**
 * @mago-expect analysis:redundant-caught-type
 */
function earlier_catch_covers_later(): string
{
    try {
        throw new EntryMissing();
    } catch (LookupFailed) {
        return 'lookup';
    } catch (EntryMissing) {
        return 'entry';
    }
}

function unrelated_siblings_are_fine(): string
{
    try {
        throw new EntryMissing();
    } catch (EntryMissing|\LogicException) {
        return 'failed';
    }
}

function narrower_earlier_catch_is_fine(): string
{
    try {
        throw new EntryMissing();
    } catch (EntryMissing) {
        return 'entry';
    } catch (LookupFailed) {
        return 'lookup';
    }
}

/**
 * @mago-expect analysis:duplicate-caught-type
 */
function exact_duplicate_still_reports_duplicate_only(): string
{
    try {
        throw new EntryMissing();
    } catch (EntryMissing|EntryMissing) {
        return 'failed';
    }
}
