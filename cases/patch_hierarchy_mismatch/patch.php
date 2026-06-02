<?php

class ClassInterfaceMismatch implements WrongIface {}

interface InterfaceExtendsMismatch extends WrongInterface {}

class ClassParentMismatch extends WrongParent {}

/** @require-extends WrongClass */
trait TraitRequireExtendsMismatch {}

/** @require-implements WrongMarker */
trait TraitRequireImplementsMismatch {}

class ClassInterfaceOmitted {}

interface InterfaceExtendsOmitted {}

class ClassParentMatch extends ActualParent {}

class ClassParentOmitted {}

trait TraitRequireExtendsOmitted {}

trait TraitRequireImplementsOmitted {}

class ClassForHierarchyMismatchMethodCheck extends WrongParent {
    public function methodFromPatch(): void {}
}
