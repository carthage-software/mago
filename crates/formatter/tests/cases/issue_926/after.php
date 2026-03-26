<?php

$user             = $this->repository->find($id);
$userEmail        = Arr::get($user->data, 'email');
$userPreferences  = Arr::get($user->data, 'preferences');
$this->name       = $user->name;
$this->status     = $user->status;
$this->created_at = $user->created_at;
