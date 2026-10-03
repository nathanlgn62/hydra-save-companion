## Buglist : Mise à jour à la fermeture, Toast de cooldown et Upload

### 1. Rafraîchissement des jeux à la fermeture pour détecter la sauvegarde locale

- **Problème / Constat :** À la fermeture d'un jeu, la liste des jeux (`games` query) n'est pas invalidée immédiatement. L'application rate visuellement la mise à jour du fichier local sur le disque mais n'empêche pas la synchronisation.
- **Comportement attendu :** Forcer le rafraîchissement des données visuelles de jeux dès la fermeture détectée pour actualiser le tag de status.
- **Tâches techniques :**
  - Localiser l'événement de fermeture du processus du jeu.
  - Appeler `queryClient.invalidateQueries({ queryKey: ["games"] })` de manière synchrone/bloquante avant d'évaluer le statut de synchronisation.
  - S'assurer que le thread attend la fin du rafraîchissement des données avant de lancer le traitement.

---

### 2. Double toast lors du déclenchement du cooldown (45s)

- **Problème / Constat :** Un toast d'avertissement lié au cooldown de 45 secondes apparaît en double à l'écran en raison d'appels multiples rapprochés ou concurrents.
- **Comportement attendu :** Un seul toast de cooldown ne doit pouvoir être émis sur une fenêtre temporelle donnée pour éviter le spam visuel dans le panneau de notifications.
- **Tâches techniques :**
  - Identifier la fonction déclenchant le message de cooldown.
  - Mettre en place un mécanisme de verrouillage par horodatage (`timestamp`) ou un verrou `useRef` pour bloquer l'émission de toasts identiques dans l'intervalle des 45 secondes.

---

### 3. Ajout d'un toast de signalement pour l'upload d'une sauvegarde

- **Problème / Constat :** L'application effectue l'envoi vers le cloud mais l'utilisateur manque de visibilité textuelle immédiate via les notifications lorsque l'upload d'une sauvegarde locale vers le cloud démarre ou se termine.
- **Comportement attendu :** Informer explicitement l'utilisateur via un toast dédié lors du déclenchement de l'upload d'une sauvegarde vers le cloud.
- **Tâches techniques :**
  - Intégrer un appel à `showToast` avec un message explicite (ex: `Upload de la sauvegarde pour ${game.title}...`) au moment de l'envoi vers le cloud.
  - Ajouter un toast de confirmation de succès ou d'erreur à la fin de la mutation d'upload.
