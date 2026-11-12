<?php

// A padded line that would pass the print width is left alone.
$options['_conditional'] = Psr7\Utils::caselessRemove(['Content-Type', 'Content-Length'], $options['_conditional']);
$options['_conditional']['Content-Type'] = 'application/x-www-form-urlencoded';

// A value that the formatter joins onto one line still aligns on the first pass.
$this->scheme   = isset($parts['scheme']) ? $this->filterScheme($parts['scheme']) : '';
$this->userInfo = isset($parts['user']) ? $this->filterUserInfoComponent($parts['user']) : '';
$this->host     = isset($parts['host']) ? $this->filterHost($parts['host']) : '';

// A value that spans lines ends its run, and keeps its normal layout.
$types = self::OUTPUT_NORMAL | self::OUTPUT_RAW | self::OUTPUT_PLAIN;
$verbosities =
    self::VERBOSITY_QUIET
    | self::VERBOSITY_NORMAL
    | self::VERBOSITY_VERBOSE
    | self::VERBOSITY_VERY_VERBOSE
    | self::VERBOSITY_DEBUG;
$verbosity = $verbosities & $options ?: self::VERBOSITY_NORMAL;
$type      = $types & $options;

// A trailing comment does not count against the print width.
$longerName = 2;
$short      = 1; // a comment that is long enough to push this whole line well past the print width of the formatter

// Compound operators line up at the operator.
$result          .= $this->origTokens->getTokenCode($pos, $itemStartPos, $indentAdjustment);
$skipRemovedNode = true;

$map = [
    Stmt\Global_::class . '->vars'                            => ', ',
    Stmt\TraitUseAdaptation\Precedence::class . '->insteadof' => ', ',
    Stmt\Unset_::class . '->vars'                             => ', ',
    'ǆ'                                                       => 'Ǆ',
    'nested'                                                  => [
        'a'   => 1,
        'bbb' => 2,
    ],
    'after'                                                   => true,
    $prefix . 'parentClassesExcludedFromUnintentionallyCoveredCodeCheckWithAnEvenLongerNameThanThat' =>
        $this->parentClassesExcludedFromUnintentionallyCoveredCodeCheck,
    'last'                                                    => false,
];

$inline = ['a' => 1, 'bbb' => 2];

final class Example
{
    public const ?array FIRST  = null;
    public const int    SECOND = 2;
    public const string THIRD  = 'a value that is long enough to make this constant line reach past the print width';

    private ?string $name  = null;
    private int     $count = 0;
    #[Deprecated]
    private int  $legacy  = 0;
    private bool $enabled = true;
}

enum Suit: string
{
    case Hearts   = 'H';
    case Spades   = 'S';
    case Diamonds = 'D';
}

$label = match ($status) {
    Status::Draft                         => 'Draft',
    Status::NeedsReview, Status::Rejected => 'Needs review',
    default                               => throw new LogicException('Unknown status: ' . $status->name),
};
