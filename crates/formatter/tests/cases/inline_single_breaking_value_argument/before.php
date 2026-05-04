<?php

DateFilter::make('affects_flights_from')
    ->icon('tabler:calendar-time')
    ->default(['start' => $now->startOfDay(), 'end' => $now->endOfDay()])
    ->supportedOperators(Operator::BETWEEN)
    ->description('Filter notices affecting flights departing or arriving between the specified dates. A notice is included if it affects at least one flight departing or arriving within the specified timeframe.');

$x = some_function_with_a_long_name_yes(12345678901234567890);
$x = some_function_with_a_long_name_yes($a_long_variable_name_that_overflows_the_print_width);
$x = some_function_with_a_long_name_yes(SomeClass::SOME_VERY_LONG_CONSTANT_NAME_HERE_XXX);

// Single complex arg (call): still breaks because the inner call is not a value.
$x = wrapper(inner_call_that_returns_a_long_value_with_a_long_method_name_here_111(), );
