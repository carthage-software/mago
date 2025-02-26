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
