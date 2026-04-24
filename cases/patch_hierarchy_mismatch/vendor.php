<?php

interface ActualIface {}

interface WrongIface {}

class ClassInterfaceMismatch implements ActualIface {}

interface ParentInterface {}

interface WrongInterface {}

interface InterfaceExtendsMismatch extends ParentInterface {}

class ActualParent {}

class WrongParent {}

class ClassParentMismatch extends ActualParent {}

class BaseClass {}

class WrongClass {}

/** @require-extends BaseClass */
trait TraitRequireExtendsMismatch {}

interface VendorMarker {}

interface WrongMarker {}

/** @require-implements VendorMarker */
trait TraitRequireImplementsMismatch {}

class ClassInterfaceOmitted implements ActualIface {}

interface InterfaceExtendsOmitted extends ParentInterface {}

class ClassParentMatch extends ActualParent {}

class ClassParentOmitted extends ActualParent {}

/** @require-extends BaseClass */
trait TraitRequireExtendsOmitted {}

/** @require-implements VendorMarker */
trait TraitRequireImplementsOmitted {}

class ClassForHierarchyMismatchMethodCheck extends ActualParent {}
