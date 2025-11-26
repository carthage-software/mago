<?php

declare(strict_types=1);

$username = '  john_doe  ';
$cleaned_username = $username
    |> trim(...)
    |> (fn($name) => strtoupper($name))
    |> (fn($name) => 'Welcome, ' . $name . '!');

$user_activation_status = $request_data
    |> get_user_id_from_request(...)
    |> (fn(?int $id) => ($id !== null ? $user_repo->findById($id) : null))
    |> function (?UserEntity $user) use ($event_dispatcher) {
        if ($user) {
            $user->activate();
            $event_dispatcher->dispatch('user.activated', ['id' => $user->id]);


            return $user->formatName();  }

        return 'User not found or activation failed.';
    }
    |> 'strval';

$numbers = [1, 2, 3, 4, 5, 6];
$processed_numbers_sum = $numbers |> (fn($x) => array_filter($x, fn($n) => $n % 2 === 0)) |> (fn($x) => array_map(fn($n) => $n * 10, $x))
    |> array_sum(...);

$app_name = $config_path
    |> file_get_contents(...)
    |> parse_config(...)
    |> (fn($parsed_config) => new Config($parsed_config))
    |> (fn(Config $c) => $c->get('appName', 'DefaultApp'));

$app_name = $raw_config
    |> (fn($data) => $data['appName'] ?? 'DefaultApp')
    |> 'strtoupper';

$user_details_name = 42
    |> UserRepository::fetchUserDetails(...)  |> 
    
    (fn($details) => $details['name']);

$is_valid_adult = $input_age |>
'intval'
    |>
    validate_age(...);

$discounted_price = $original_price
|> (fn($x) => calculate_discount($x, 10.0)) |>
(fn($price) => round($price, 2));

$extracted_value = $data_packet |> (fn($x) => explode(',', $x)) |> (fn($parts) => array_reduce(
        $parts,
        function ($carry, $part) {
            [$k, $v] = explode(':', $part);
            $carry[$k] = $v;


            return $carry;
        },
        []
    ))
    |> (fn($assoc_array) => (float)($assoc_array['value'] ?? 0.0));

$initial_value = '  start  ';
$complex_pipe_result = $initial_value |> 'trim' |> (fn($trimmed) => $trimmed  |> strtoupper(...))  |>
    (fn($uppercased) => ('PREFIX_' . $uppercased . '_SUFFIX')) |> (fn($final) => str_replace('PREFIX_', '', $final));

$user_object = $user_id |> $user_repo->findById(...) // Find user by ID
    |>
    function(?UserEntity $user): UserEntity { if ($user === null) { return new UserEntity(0, 'Guest', 'guest@example.com'); }
       
        return $user; } |>
   ( fn(UserEntity $u) => $u->activate());
