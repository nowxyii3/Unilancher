# UniLauncher 🎮

**UniLauncher** est un launcher de jeux PC universel, moderne et ultra-performant écrit en Rust avec le framework graphique natif [egui / eframe](https://github.com/emilk/egui). Il centralise l'ensemble de vos jeux PC (Steam, Epic Games, Xbox, GOG, Ubisoft, Battle.net, EA, émulateurs et jeux manuels) dans une interface gaming sombre, fluide et sans distraction.

---

## ✨ Nouveautés majeures — Version 1.1.0

### 🎨 Refonte complète de la Bibliothèque de jeux
- **Cartes de jeu modernes et réactives** : Jaquettes au ratio gaming standardisé avec bordures lumineuses et ombrages subtils au survol.
- **Actions rapides au survol (Hover Overlay)** : Bouton « Jouer » direct, accès rapide à la fiche détaillée et bascule Favori instantanée.
- **Tailles de grille adaptatives (S / M / L)** : Choisissez entre Petites (148×210px), Moyennes (184×258px) ou Grandes (228×318px) cartes avec préservation de l'échelle personnalisée.
- **Mode Liste immersif enrichi** : Vue tabulaire haute lisibilité affichant le statut d'installation, la plateforme, le temps de jeu cumulé, la date de la dernière session et les actions rapides.
- **Optimisation Viewport Culling** : Rendu dynamique n'exécutant que les cartes et lignes visibles à l'écran, garantissant un framerate constant même sur de gigantesques bibliothèques (+1 000 jeux).

### 🔍 Filtres avancés & Multi-tris combinables
- **Barre de filtres d'état segmentée** :
  - *Tous les jeux*
  - *⭐ Favoris*
  - *🕒 Récents* (jeux avec temps de jeu ou session récente)
  - *✨ Jamais lancés*
  - *🟢 Installés*
  - *⚠️ Non installés / Introuvables*
- **6 modes de tri avancés** :
  - Temps de jeu (décroissant)
  - Nom (A → Z)
  - Nom (Z → A)
  - Dernière utilisation
  - Favoris en premier
  - Taille sur le disque (décroissant)

### 🌟 Fiche de jeu immersive intégrée (Pleine page)
Remplacement de l'ancienne fenêtre modale flottante par une page immersive dédiée parfaitement intégrée :
- **Bannière Hero haute définition** : Prise en charge des bannières locales Steam, bannières personnalisées utilisateur ou génération procédurale d'un dégradé gaming adapté aux couleurs dominantes du jeu.
- **En-tête dynamique** : Jaquette incrustée, titre grand format, badge de plateforme stylisé, temps de jeu précis, date de dernière session et boutons d'action rapide (Jouer, Favori, Ouvrir le dossier, Sauvegarde).
- **Navigation par 5 onglets thématiques** :
  1. **Vue d'ensemble** : Statut d'installation, plateforme, taille sur disque estimée, chemin d'accès, date de sortie et description.
  2. **Statistiques** : Temps de jeu, temps de jeu moyen par session, date du dernier lancement, historique et estimation du coût par heure jouée.
  3. **Captures d'écran** : Galerie de screenshots avec ouverture en visionneuse haute résolution (Lightbox) et lien direct vers le dossier de captures.
  4. **Succès & Trophées** : Barre de progression globale avec pourcentage, compteur débloqué/total, et filtre instantané (Tous / Débloqués / Verrouillés).
  5. **Paramètres du jeu** : Personnalisation de la bannière et de la jaquette (sélecteur de fichier image ou réinitialisation), arguments de lancement (CLI), renommage local, option pour masquer le jeu et désinstallation.

### 📊 Correction définitive des superpositions dans les Statistiques
- Refonte de la mise en page des listes **« Top 5 les plus joués »** et **« Top 5 les plus volumineux »** avec allocation stricte de colonnes et découpage intelligent (`trunc`) garantissant une mise en page parfaite sans chevauchement de texte quel que soit le niveau de mise à l'échelle DPI de Windows (100%, 125%, 150%).
- Harmonisation visuelle des jauges de distribution par plateforme.

---

## 🛠️ Stack technique & Architecture

- **Langage** : Rust (Édition 2021)
- **Framework UI** : [egui 0.31 / eframe](https://github.com/emilk/egui) — Rendu matériel GPU via Glow / WGPU
- **Système de packaging & installation** : Inno Setup 6 (`packaging/unilauncher.iss`)
- **CI / CD** : GitHub Actions (`.github/workflows/build-and-release.yml`)
- **Stockage local des données** : `%APPDATA%\UniLauncher\`
  - `config.json` : Paramètres, favoris, filtres mémorisés, bannières et jaquettes personnalisées
  - `playtime.json` : Enregistrement continu des sessions de jeu
  - `banners_cache/` : Cache local des visuels haute définition

---

## 🚀 Compilation & Développement

### Prérequis
- [Rust & Cargo](https://rustup.rs/) (dernière version stable recommandée)
- Windows 10 ou 11 (64-bit)

### Commandes utiles

```powershell
# Vérification rapide de la compilation
cargo check

# Lancer la suite complète de tests unitaires
cargo test

# Lancer l'application en mode développement
cargo run

# Compiler la version de production optimisée
cargo build --release
```

Le binaire optimisé est généré dans `target/release/UniLauncher.exe`.

---

## 📄 Licence

Ce projet est sous licence MIT. Consultez le fichier [LICENSE](LICENSE) pour plus de détails.
