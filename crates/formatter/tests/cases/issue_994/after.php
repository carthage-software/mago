<?php

class X
{
    /**
     * @param TaxRuleConfigArray $record
     * @return TaxRule
     */
    public static function fromConfigArray(array $record): TaxRule
    {
        return new TaxRule(
            id: TaxRuleEntityId::from($record['id']),
            name: $record['type']->value,
            country_code_regex: $record['regex'],
            tax_rate: $record['tax_rate'],
            taxRuleType: $record['type'],
        );
    }

    /**
     * @return array{id:string,name:string,regex:string,tax_rate:float,type:TaxRuleType}
     */
    public function toConfigArray(): array
    {
        return [
            'id' => $this->getId()->getValue(),
            'name' => $this->getName(),
            'regex' => $this->getCountryCodeRegex(),
            'tax_rate' => $this->getTaxRateFloat(),
            'type' => $this->getTaxRuleType(),
        ];
    }
}
