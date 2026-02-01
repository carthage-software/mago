<?php

$workingOrders = [
    ['taskName' => 'Task 1', 'requiredEmployeeCount' => 2],
    ['taskName' => 'Task 2'] // Missing requiredEmployeeCount
];

foreach ($workingOrders as $workingOrder) {
    if (!empty($workingOrder['requiredEmployeeCount'])) {
        echo $workingOrder['requiredEmployeeCount'];
    }
}
