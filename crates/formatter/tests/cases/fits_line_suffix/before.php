<?php

// Chain with trailing comment on intermediate call
$form
    ->setMethod('get')
    ->setTitle($this->getPageTitle()) // Remove subpage
    ->setFormIdentifier('blocklist')
    ->setWrapperLegendMsg('legend')
    ->setSubmitTextMsg('submit')
    ->prepareForm()
    ->displayForm(false);

// Chain with trailing comment on last call
$form
    ->setMethod('get')
    ->setTitle($this->getPageTitle())
    ->setFormIdentifier('blocklist')
    ->setWrapperLegendMsg('legend')
    ->setSubmitTextMsg('submit')
    ->prepareForm()
    ->displayForm(false); // Remove subpage

// Short chain with trailing comment (should stay on one line)
$result = $builder->method($a)->other(); // trailing

// Argument list with intermediate trailing comment
foo(
    $first, // after first
    $second,
    $third,
);

// Binary chain with trailing comment in if condition
if (
    $condition_a
    && $condition_b // check b
    && $condition_c
    && $condition_d
) {
    return true;
}
?>
<div><?php echo $value; /* inline block comment */ ?>text</div>
