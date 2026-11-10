# Audit initial de ScreenTranslate

## Révision examinée

Fork local propre sur `main`, commit `e6e6dc850a34a6884db9970c5a5d58a0ddbd9966` (après v1.6.0). Le HEAD du [dépôt amont](https://github.com/hcmhcs/screenTranslate) correspond à cette révision. Le fork sert de base pour conserver son historique.

## Inventaire

Le dépôt contient 101 fichiers Swift, un projet Xcode, des ressources et des scripts de publication. Le seul fichier Python, `scripts/update_appcast.py`, génère le flux de mise à jour Sparkle. Il ne s’exécute pas dans l’application. La migration porte donc sur du code Swift, et la distribution peut se passer du script Python.

| Fonction présente dans le code | Implémentation | Décision |
| --- | --- | --- |
| Capture rectangulaire et suivi continu d’une région | AppKit, ScreenCaptureKit, deux chemins de capture | Conserver les adaptateurs natifs et leurs règles de coordonnées Retina |
| OCR avec langue détectée et prétraitement | Vision, NaturalLanguage, `OCRProvider` | Conserver Vision, ajouter les observations géométriques sans casser les consommateurs existants |
| Traduction de région, texte sélectionné, presse-papiers, saisie et sous-titres bêta | Coordinateur Swift, Apple Translation et moteurs DeepL, Google, Azure | Réutiliser les contrats, garder Apple Translation dans son contexte UI natif |
| Fenêtre flottante et aperçu du texte source | AppKit, SwiftUI | Réutiliser le résultat rapide avant de créer les autres présentations |
| Historique des traductions | SwiftData, avec repli mémoire | Conserver les données existantes. Créer un catalogue documentaire séparé avant de migrer |
| Raccourcis, ouverture à la connexion et gestion des permissions | KeyboardShortcuts, ServiceManagement, Accessibility, ScreenCaptureKit | Conserver ces points d’entrée |
| Clés API | Security / Keychain | Conserver l’accès natif et vérifier les flux réseau avant migration |
| Mises à jour et mesures anonymes | Sparkle, TelemetryDeck | Séparer la distribution du fork de l’amont et supprimer les envois de télémétrie |

Cet inventaire décrit les chemins trouvés dans les sources. Il ne constitue pas un test manuel. Aucun éditeur non destructif, catalogue de captures, menu circulaire ni export Markdown/Obsidian n’a été trouvé. La localisation existante couvre l’anglais et le coréen via `L10n`, avec quelques chaînes et valeurs par défaut ailleurs dans le projet.

## Licences et dépendances

- Application et modifications distribuées sous GPL-3.0, avec notices et sources correspondantes. Conserver `LICENSE`.
- [KeyboardShortcuts](https://github.com/sindresorhus/KeyboardShortcuts) sous MIT et [Sparkle](https://github.com/sparkle-project/Sparkle) sous licence de type MIT, avec notices tierces dans le dépôt. Le projet impose des minimums 2.0.0 et 2.9.0.
- [TelemetryDeck SwiftSDK](https://github.com/TelemetryDeck/SwiftSDK) est référencé à partir de 2.11.0. Son retrait évite toute dépendance à ce SDK dans le fork.
- Le catalogue de polices référence des téléchargements facultatifs. La police Pretendard intégrée est publiée sous SIL OFL 1.1. Vérifier la présence des notices de la police et des composants tiers avant distribution publique.
- Les frameworks Apple sont fournis par macOS. Vérifier disponibilité et signatures avec le SDK cible. L’amont fixe macOS 15 et documente Xcode 26 ou ultérieur.

## Migration et non-régression

Garder l’application existante comme point d’entrée. Ajouter d’abord un moteur Rust autonome pour l’export, compilé en bibliothèque statique et appelé via une frontière C étroite. Les images restent dans le processus et sur le disque local. Les permissions, fenêtres, OCR, traduction Apple et dialogues de fichiers restent dans les adaptateurs Swift. Étendre ensuite le moteur aux annotations, règles d’automatisation et index de bibliothèque.

Exécuter les tests existants de coordination et d’annulation, OCR Vision, moteurs HTTP simulés, historique, préférences, localisation et traduction continue. Vérifier séparément en session graphique les raccourcis globaux, les permissions refusées puis accordées, la capture sur deux écrans dont un Retina, les packs Apple et la restauration du presse-papiers. Tester l’export Rust avec texte Unicode, liens relatifs, erreurs de disque et destinations existantes. Vérifier l’ouverture de l’application arm64 et l’absence d’envoi réseau au démarrage avant toute publication.

Le premier lancement de `xcodebuild test` dans l’environnement restreint a échoué à la résolution des packages GitHub. Il faut relancer la référence avant d’interpréter les résultats de tests de la refonte.
