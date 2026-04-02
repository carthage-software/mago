<?php

expect(AuditLog::query()->sole()->data)->toBeInstanceOf(UserLoggedIn::class);
expect(AuditLog::query()->sole()->data->user_id)->toBe(99);

$this->tokenStorage->getToken()->getUser()->getFoo();
