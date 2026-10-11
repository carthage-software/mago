<?php

$this->assertFalse(collect(DB::connection()->getQueryLog())->contains(
  fn (array $query): bool => str_contains($query['query'], 'campaign_item_job'),
));
