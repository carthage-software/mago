<input type="text" class="class_name" value="<?=
    isset($object->long_property) ? htmlspecialchars($object->long_property) : ''
?>" name="objects[<?= htmlspecialchars($object->other_property) ?>][long_property]" placeholder="<? echo $placeholder ?>">
