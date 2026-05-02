+++
title = "Guard"
description = "Ce que fait le guard et comment ses deux moitiés divisent le travail."
nav_order = 10
nav_section = "Tools"
nav_subsection = "Guard"
+++
# Guard

`mago guard` applique les frontières architecturales et les conventions structurelles à travers un projet PHP. Il couvre le même terrain que deptrac et arkitect, dans un seul binaire fonctionnant sur le parseur de Mago.

L'outil a deux moitiés : le guard de périmètre valide les dépendances entre couches, et le guard structurel applique les conventions sur les symboles eux-mêmes.

## Guard de périmètre

Le guard de périmètre valide les arêtes de dépendance. Il garantit que différentes parties d'une application ne se parlent que de manières que vous avez explicitement autorisées, de sorte que le domaine reste libre de fuites d'infrastructure et que l'UI ne puisse pas atteindre au-delà de la couche application.

Règles typiques :

- La couche `Domain` ne doit dépendre d'aucune autre couche.
- La couche `UI` peut dépendre de `Application` mais pas l'inverse.
- Un module spécifique n'est autorisé à utiliser qu'une liste approuvée de bibliothèques.

## Guard structurel

Le guard structurel applique les conventions sur les symboles eux-mêmes : leurs noms, modificateurs, super-types, attributs et la forme de leur espace de noms contenant.

Règles typiques :

- Toutes les classes dans `App\Http\Controllers` doivent être `final` et se terminer par `Controller`.
- Les interfaces sous `Domain` doivent se terminer par `Interface`.
- Un espace de noms spécifique ne peut contenir que des définitions `enum`.

## Où aller ensuite

- [Utilisation](/tools/guard/usage/) : les commandes courantes et à quoi ressemble leur sortie.
- [Référence de commande](/tools/guard/command-reference/) : tous les indicateurs que `mago guard` accepte.
- [Référence de configuration](/tools/guard/configuration-reference/) : toutes les options que Mago accepte sous `[guard]`.
