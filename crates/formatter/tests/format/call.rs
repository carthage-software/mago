use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

#[test]
pub fn test_class_member_access_chain() {
    let code = indoc! {r#"
        <?php

        expect($response->dto())->toBeInstanceOf(QuoteData::class)->aircraft_type->toBe($aircraftType)->status->toBe($status)
            ->customer_id->toBe($customerId)->account_id->toBe($accountId)->salesperson_id->toBe($salespersonId)->price
            ->toBe($price)->currency->toBe('EUR')->id->toBe($id);
    "#};

    let expected = indoc! {r#"
        <?php

        expect($response->dto())
            ->toBeInstanceOf(QuoteData::class)
            ->aircraft_type->toBe($aircraftType)
            ->status->toBe($status)
            ->customer_id->toBe($customerId)
            ->account_id->toBe($accountId)
            ->salesperson_id->toBe($salespersonId)
            ->price->toBe($price)
            ->currency->toBe('EUR')
            ->id->toBe($id);
    "#};

    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_chain_inside_conditional() {
    let code = indoc! {r#"
        <?php

        $interval = $report->finished_at
          ? CarbonInterval::make($report->finished_at->diffInSeconds($report->created_at), 'seconds',
          )->cascade()->cascade()->forHumans(['parts' => 1, 'options' => Carbon::CEIL])
          : null;
    "#};

    let expected = indoc! {r#"
        <?php

        $interval = $report->finished_at
            ? CarbonInterval::make($report->finished_at->diffInSeconds($report->created_at), 'seconds')
                ->cascade()
                ->cascade()
                ->forHumans(['parts' => 1, 'options' => Carbon::CEIL])
            : null;
    "#};

    test_format(code, expected, FormatSettings { ..FormatSettings::default() });
}
