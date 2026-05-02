+++
title = "Recette pre-commit hooks"
description = "Lancer lint, analyze et format sur les fichiers indexés avant chaque commit."
nav_order = 40
nav_section = "Recettes"
+++
# Recette pre-commit hooks

Exécuter Mago automatiquement avant chaque commit git. Les exemples ci-dessous agissent tous uniquement sur les fichiers indexés, donc le hook reste rapide même sur des dépôts volumineux.

## Configuration rapide

Créez `.git/hooks/pre-commit` et rendez-le exécutable :

```bash
chmod +x .git/hooks/pre-commit
```

## Configurations de hook

Choisissez la configuration qui correspond à votre workflow.

### Auto-formater les fichiers indexés

Formater les fichiers PHP indexés et ré-indexer les versions formatées. L'expérience la plus fluide pour les développeurs ; rien à se rappeler.

```bash
#!/bin/sh

mago lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --staged
if [ $? -ne 0 ]; then
    echo "Formatting failed. Please check the error above."
    exit 1
fi

exit 0
```

`--staged` trouve les fichiers indexés et ne traite que ceux-là. Pour `fmt --staged`, les fichiers formatés sont automatiquement ré-indexés. Pour `lint --staged` et `analyze --staged`, combinés à `--fix`, les fichiers corrigés sont ré-indexés.

### Auto-corriger et auto-formater les fichiers indexés

Cela ajoute `--fix` à l'étape lint. `--fail-on-remaining` bloque le commit si des problèmes n'ont pas pu être corrigés automatiquement et nécessitent encore une attention manuelle. Sans cela, `--fix` quitte avec zéro même quand des problèmes non corrigés subsistent.

```bash
#!/bin/sh

mago lint --fix --fail-on-remaining --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the remaining issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --staged
if [ $? -ne 0 ]; then
    echo "Formatting failed. Please check the error above."
    exit 1
fi

exit 0
```

Pour des corrections plus agressives, utilisez `--fix --unsafe` ou `--fix --potentially-unsafe` :

```bash
mago lint --fix --potentially-unsafe --fail-on-remaining --staged
```

### Bloquer les commits quand le formatage diverge

Refuser le commit si un fichier indexé n'est pas correctement formaté, obligeant le développeur à formater manuellement.

```bash
#!/bin/sh

mago lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --check
if [ $? -ne 0 ]; then
    echo "Some files are not formatted. Please run 'mago fmt' before committing."
    exit 1
fi

exit 0
```

## Husky

Si vous utilisez [Husky](https://typicode.github.io/husky/), ajoutez les commandes à `.husky/pre-commit` :

```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

mago lint --staged
mago analyze --staged
mago fmt --staged
```

## CaptainHook

Si vous utilisez [CaptainHook](https://docs.captainhook.info/), ajoutez les actions à `captainhook.json` :

```json
{
    "pre-commit": {
        "enabled": true,
        "actions": [
            { "action": "mago lint --staged" },
            { "action": "mago analyze --staged" },
            { "action": "mago fmt --staged" }
        ]
    }
}
```

Pour la variante check-only, remplacez la dernière action par `mago fmt --check`.

## `--staged` versus `--check`

| Aspect | `--staged` | `--check` |
| :--- | :--- | :--- |
| Comportement | Formate les fichiers indexés et les ré-indexe. | Signale les fichiers non formatés ; échoue s'il y en a. |
| Action développeur | Aucune. | Doit lancer `mago fmt` manuellement si la vérification échoue. |
| Idéal pour | Équipes qui veulent un formatage transparent. | Équipes qui veulent un contrôle explicite des changements. |
| Indexation partielle | Formate le contenu indexé, laisse l'arbre de travail tranquille. | Fonctionne quel que soit l'état d'indexation. |
