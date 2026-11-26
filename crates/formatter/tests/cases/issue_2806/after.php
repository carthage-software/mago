<?php switch ($t) {
    case 1:
        ?>
    <p>one</p>
<?php break; ?>
<?php
    default:
        ?>
    <p>other</p>
<?php
} ?>
<?php switch ($t):
    case 'a':
        ?>x<?php
endswitch;
