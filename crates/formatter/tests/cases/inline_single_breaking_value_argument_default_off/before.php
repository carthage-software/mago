<?php

DateFilter::make('affects_flights_from')
    ->icon('tabler:calendar-time')
    ->default(['start' => $now->startOfDay(), 'end' => $now->endOfDay()])
    ->supportedOperators(Operator::BETWEEN)
    ->description('Filter notices affecting flights departing or arriving between the specified dates. A notice is included if it affects at least one flight departing or arriving within the specified timeframe.');
