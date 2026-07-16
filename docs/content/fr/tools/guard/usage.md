+++
title = "Utilisation du guard"
description = "Exécuter mago guard et lire ce qu'il signale."
nav_order = 20
nav_section = "Tools"
nav_subsection = "Guard"
+++
# Utilisation du guard

Pour vérifier le projet par rapport aux règles dans `mago.toml` :

```sh
mago guard
```

Pour vérifier un seul répertoire ou fichier :

```sh
mago guard src/Domain
mago guard src/UI/Controller/UserController.php
```

Les chemins passés sur la ligne de commande remplacent les `paths` de `mago.toml` pour cette exécution.

## Lecture de la sortie

Le guard signale deux types de problèmes : les violations de frontière du guard de périmètre et les défauts structurels du guard structurel.

### Violation de frontière

Étant donné cette règle :

```toml
[[guard.perimeter.rules]]
namespace = "App\\Domain\\"
permit = ["@self", "@native"]
```

Et ce code :

```php
namespace App\Domain\Model;

use App\Infrastructure\Doctrine\Orm\Entity;

class User extends Entity {}
```

Le guard signale :

```
error[disallowed-use]: Illegal dependency on `App\Infrastructure\Doctrine\Orm\Entity`
 ┌─ src/Domain/Model/User.php:4:5
 │
4 │ use App\Infrastructure\Doctrine\Orm\Entity;
 │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ This `use` statement is not allowed by the architectural rules
 │
 = Breach occurred in namespace `App\Domain\Model`.
 = Dependency forbidden by architectural rules
 = The following rule(s) were evaluated but none permitted this dependency: `App\Domain\\`.
 = Help: Update your guard configuration to allow this dependency or refactor the code to remove it.
```

### Restreindre les endroits où une dépendance peut être utilisée

Les restrictions de périmètre protègent une dépendance vis-à-vis de certains espaces de noms sources. Cette règle autorise l'utilisation du contrôleur de base uniquement par le code sous `App\Http\Controllers\` :

```toml
[[guard.perimeter.restrictions]]
dependency = "App\\Http\\Controllers\\Controller"
allow-from = ["App\\Http\\Controllers\\"]
```

Cette règle interdit un trait externe dans tout `App\`, même si une règle de périmètre ordinaire l'autorise :

```toml
[[guard.perimeter.restrictions]]
dependency = "Illuminate\\Foundation\\Bus\\Dispatchable"
deny-from = ["App\\"]
```

Les restrictions conviennent aux interdictions ciblées. Lorsqu'aucune règle de périmètre ordinaire ni aucun layering n'est configuré, les dépendances qui ne correspondent à aucune restriction restent autorisées.

### Défaut structurel

Étant donné cette règle :

```toml
[[guard.structural.rules]]
on = "App\\UI\\**\\Controller\\**"
target = "class"
must-be-final = true
reason = "Controllers should be final to prevent extension."
```

Et ce code :

```php
namespace App\UI\Controller;

class UserController
{
}
```

Le guard signale :

```
error[must-be-final]: Structural flaw in `App\UI\Controller\UserController`
 ┌─ src/UI/Controller/UserController.php:3:7
 │
 3 │ class UserController
 │ ^^^^^^^^^^^^^^ This must be declared as `final`
 │
 = Controllers should be final to prevent extension.
 = Help: Declare this class as `final`.
```

Chaque rapport identifie le symbole, l'emplacement, la violation exacte et la `reason` de la configuration lorsqu'elle a été fournie.

### Limiter les méthodes publiques

Utilisez `only-public-methods` pour limiter l'API publique déclarée directement par les classes correspondantes :

```toml
[[guard.structural.rules]]
on = "App\\Http\\Controllers\\**"
target = "class"
only-public-methods = ["__construct", "__invoke"]
```

Toute autre méthode publique déclarée directement produit un défaut `only-public-methods`. Les méthodes privées et protégées restent autorisées. Les noms configurés sont autorisés, mais ne sont pas obligatoires, et les méthodes héritées ou fournies par des traits ne sont pas vérifiées.
