# Guide de Contribution

Merci de votre intérêt pour contribuer à DeepVault ! Ce document fournit des directives pour contribuer au projet.

## Comment Contribuer

### Signaler un Bug

1. Vérifiez que le bug n'a pas déjà été signalé dans les [Issues](https://github.com/deepvault/usb-deepvault/issues)
2. Créez une nouvelle issue avec le label "bug"
3. Incluez :
   - Description détaillée du problème
   - Étapes pour reproduire
   - Version du système d'exploitation
   - Logs d'erreur si disponibles

### Proposer une Fonctionnalité

1. Vérifiez que la fonctionnalité n'a pas déjà été proposée
2. Créez une issue avec le label "enhancement"
3. Décrivez :
   - Le cas d'usage
   - L'impact sur la sécurité
   - La complexité d'implémentation

### Contribution de Code

1. **Fork** le repository
2. **Clone** votre fork localement
3. Créez une **branche feature** : `git checkout -b feature/ma-fonctionnalite`
4. **Commitez** vos changements : `git commit -m 'Ajout: description de la fonctionnalité'`
5. **Push** vers votre fork : `git push origin feature/ma-fonctionnalite`
6. Créez une **Pull Request**

## Standards de Code

### Rust
- Suivez les conventions Rust standard
- Utilisez `cargo fmt` pour le formatage
- Utilisez `cargo clippy` pour les suggestions
- Ajoutez des tests unitaires pour les nouvelles fonctionnalités
- Documentez les fonctions publiques

### Vue.js/JavaScript
- Suivez les conventions Vue.js 3
- Utilisez des composants fonctionnels avec Composition API
- Ajoutez des commentaires pour la logique complexe
- Respectez le style sombre de l'interface

## Tests

### Tests Unitaires
```bash
cargo test
```

### Tests d'Intégration
```bash
cargo test --test integration
```

### Tests de Sécurité
- Testez toujours sur des données non-critiques
- Vérifiez que les clés ne sont pas exposées dans les logs
- Validez l'effacement sécurisé des données temporaires

## Sécurité

### Règles Importantes
- **JAMAIS** commiter de clés ou mots de passe
- **TOUJOURS** valider les entrées utilisateur
- **TOUJOURS** effacer les données sensibles de la mémoire
- **TOUJOURS** utiliser des algorithmes cryptographiques éprouvés

### Audit de Code
- Toute modification touchant la cryptographie doit être revue
- Les changements d'API doivent être documentés
- Les tests de sécurité doivent passer

## Processus de Review

1. **Automated Checks** : CI/CD doit passer
2. **Code Review** : Au moins 2 approbations requises
3. **Security Review** : Pour les changements sensibles
4. **Testing** : Tests manuels sur différentes plateformes

## Communication

- **Issues** : Pour les bugs et demandes de fonctionnalités
- **Discussions** : Pour les questions générales
- **Security** : Pour les vulnérabilités, contactez security@deepvault.dev

## Licence

En contribuant, vous acceptez que vos contributions soient sous licence MIT.
