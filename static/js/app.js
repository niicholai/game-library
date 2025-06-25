// SH Game Hub - Main Application JavaScript
class GameHub {
    constructor() {
        this.apiBase = 'http://localhost:3000/api';
        this.games = [];
        this.currentFilter = 'all';
        this.currentSearchQuery = '';
        this.currentSection = 'library';
        this.init().catch(error => console.error('Initialization failed:', error));
    }

    async init() {
        this.bindEvents();
        await this.loadGames();
        this.updateStats();
    }

    bindEvents() {
        document.querySelectorAll('.nav-link').forEach(link => {
            link.addEventListener('click', (event) => {
                event.preventDefault();
                this.switchSection(link.dataset.section);
            });
        });

        document.querySelectorAll('.view-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                this.switchView(btn.dataset.view);
            });
        });

        document.querySelectorAll('.category-item').forEach(item => {
            item.addEventListener('click', () => {
                this.filterGames(item.dataset.filter);
            });
        });

        const searchInput = document.getElementById('searchInput');
        if (searchInput) {
            searchInput.addEventListener('input', (event) => {
                this.searchGames(event.target.value);
            });
        }

        const addGameBtn = document.getElementById('addGameBtn');
        if (addGameBtn) {
            addGameBtn.addEventListener('click', () => {
                this.showAddGameModal();
            });
        }

        const addFirstGameBtn = document.getElementById('addFirstGameBtn');
        if (addFirstGameBtn) {
            addFirstGameBtn.addEventListener('click', () => {
                this.showAddGameModal();
            });
        }

        const addGameModalClose = document.getElementById('addGameModalClose');
        if (addGameModalClose) {
            addGameModalClose.addEventListener('click', () => {
                this.hideAddGameModal();
            });
        }

        const cancelAddGame = document.getElementById('cancelAddGame');
        if (cancelAddGame) {
            cancelAddGame.addEventListener('click', () => {
                this.hideAddGameModal();
            });
        }

        const addGameForm = document.getElementById('addGameForm');
        if (addGameForm) {
            addGameForm.addEventListener('submit', async (event) => {
                event.preventDefault();
                await this.addGame();
            });
        }

        const storeSearchBtn = document.getElementById('storeSearchBtn');
        if (storeSearchBtn) {
            storeSearchBtn.addEventListener('click', async () => {
                await this.searchStore();
            });
        }

        const storeSearchInput = document.getElementById('storeSearchInput');
        if (storeSearchInput) {
            storeSearchInput.addEventListener('keypress', async (event) => {
                if (event.key === 'Enter') {
                    event.preventDefault();
                    await this.searchStore();
                }
            });
        }

        const refreshBtn = document.getElementById('refreshBtn');
        if (refreshBtn) {
            refreshBtn.addEventListener('click', async () => {
                await this.refreshLibrary();
            });
        }

        document.querySelectorAll('.modal').forEach(modal => {
            modal.addEventListener('click', (event) => {
                if (event.target === modal) {
                    modal.classList.remove('active');
                }
            });
        });

        const modalClose = document.getElementById('modalClose');
        if (modalClose) {
            modalClose.addEventListener('click', () => {
                const gameModal = document.getElementById('gameModal');
                if (gameModal) {
                    gameModal.classList.remove('active');
                }
            });
        }
    }

    async apiCall(endpoint, options = {}) {
        const response = await fetch(`${this.apiBase}${endpoint}`, {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            },
            ...options
        });

        if (!response.ok) {
            const error = new Error(`HTTP error! status: ${response.status}`);
            console.error('API call failed:', error);
            this.showNotification('API call failed: ' + error.message, 'error');
            throw error;
        }

        return await response.json();
    }

    async loadGames() {
        this.showLoading(true);
        try {
            const response = await this.apiCall('/games');
            if (response.success) {
                this.games = response.data.games;
                this.renderGames();
                this.updateStats();
                this.updateCategoryCounts();
            }
        } catch (error) {
            console.error('Failed to load games:', error);
        } finally {
            this.showLoading(false);
        }
    }

    async addGame() {
        const form = document.getElementById('addGameForm');
        if (!form) return;

        const formData = new FormData(form);
        const name = formData.get('name');
        const igdbIdRaw = formData.get('igdb_id');
        const filePathRaw = formData.get('file_path');

        const gameData = {
            name: typeof name === 'string' ? name : '',
            igdb_id: (typeof igdbIdRaw === 'string' && igdbIdRaw.trim() !== '') ? parseInt(igdbIdRaw) : null,
            file_path: (typeof filePathRaw === 'string' && filePathRaw.trim() !== '') ? filePathRaw : null
        };

        try {
            const response = await this.apiCall('/games', {
                method: 'POST',
                body: JSON.stringify(gameData)
            });

            if (response.success) {
                this.showNotification('Game added successfully!', 'success');
                this.hideAddGameModal();

                if (gameData.igdb_id) {
                    await this.fetchGameMetadata(response.data.id);
                }

                await this.loadGames();
            }
        } catch (error) {
            this.showNotification('Failed to add game', 'error');
        }
    }

    async fetchGameMetadata(gameId) {
        try {
            const response = await this.apiCall(`/games/${gameId}/metadata`, {
                method: 'POST'
            });

            if (response.success) {
                this.showNotification('Game metadata updated!', 'success');
                return response.data;
            }
        } catch (error) {
            console.error('Failed to fetch metadata:', error);
            this.showNotification('Failed to fetch game metadata', 'warning');
        }
    }

    async searchStore() {
        const queryInput = document.getElementById('storeSearchInput');
        if (!queryInput) return;
        const query = queryInput.value.trim();
        if (!query) return;

        this.showStoreLoading(true);
        try {
            const response = await this.apiCall(`/search/igdb?q=${encodeURIComponent(query)}&limit=12`);
            if (response.success) {
                this.renderStoreResults(response.data);
            }
        } catch (error) {
            this.showNotification('Store search failed', 'error');
        } finally {
            this.showStoreLoading(false);
        }
    }

    async installGame(gameId) {
        console.log('Install game:', gameId);
        this.showNotification('Install functionality coming soon!', 'info');
    }

    async uninstallGame(gameId) {
        console.log('Uninstall game:', gameId);
        this.showNotification('Uninstall functionality coming soon!', 'info');
    }

    switchSection(section) {
        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.toggle('active', link.dataset.section === section);
        });

        document.querySelectorAll('.section').forEach(sec => {
            sec.classList.toggle('active', sec.id === `${section}-section`);
        });

        this.currentSection = section;

        if (section === 'library') {
            this.loadGames().catch(error => console.error('Failed to load games:', error));
        }
    }

    switchView(view) {
        document.querySelectorAll('.view-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.view === view);
        });

        const container = document.getElementById('gamesContainer');
        if (container) {
            container.className = `games-container ${view}-view`;
        }
    }

    filterGames(filter) {
        document.querySelectorAll('.category-item').forEach(item => {
            item.classList.toggle('active', item.dataset.filter === filter);
        });

        this.currentFilter = filter;
        this.renderGames();
    }

    searchGames(query) {
        this.currentSearchQuery = query.toLowerCase();
        this.renderGames();
    }

    renderGames() {
        const container = document.getElementById('gamesContainer');
        const emptyState = document.getElementById('emptyState');
        if (!container || !emptyState) return;

        let filteredGames = this.games;

        if (this.currentFilter === 'installed') {
            filteredGames = filteredGames.filter(game => Boolean(game['is_installed']));
        } else if (this.currentFilter === 'uninstalled') {
            filteredGames = filteredGames.filter(game => !Boolean(game['is_installed']));
        }

        if (this.currentSearchQuery) {
            filteredGames = filteredGames.filter(game => {
                const nameMatch = game.name && game.name.toLowerCase().includes(this.currentSearchQuery);
                const summaryMatch = game['summary'] && game['summary'].toLowerCase().includes(this.currentSearchQuery);
                return nameMatch || summaryMatch;
            });
        }

        if (filteredGames.length === 0) {
            container.style.display = 'none';
            emptyState.style.display = 'block';
            return;
        }

        container.style.display = 'grid';
        emptyState.style.display = 'none';

        container.innerHTML = filteredGames.map(game => this.createGameCard(game)).join('');

        // Bind events after creating the HTML
        this.bindGameCardEvents(container);
    }

    bindGameCardEvents(container) {
        container.querySelectorAll('.game-card').forEach(card => {
            card.addEventListener('click', (event) => {
                if (!event.target.closest('.game-actions')) {
                    this.showGameDetails(card.dataset.gameId || '').catch(error =>
                        console.error('Failed to show game details:', error)
                    );
                }
            });
        });

        container.querySelectorAll('.btn-install').forEach(btn => {
            btn.addEventListener('click', (event) => {
                event.stopPropagation();
                this.installGame(btn.dataset.gameId || '').catch(error =>
                    console.error('Failed to install game:', error)
                );
            });
        });

        container.querySelectorAll('.btn-uninstall').forEach(btn => {
            btn.addEventListener('click', (event) => {
                event.stopPropagation();
                this.uninstallGame(btn.dataset.gameId || '').catch(error =>
                    console.error('Failed to uninstall game:', error)
                );
            });
        });

        container.querySelectorAll('.btn-details').forEach(btn => {
            btn.addEventListener('click', (event) => {
                event.stopPropagation();
                this.showGameDetails(btn.dataset.gameId || '').catch(error =>
                    console.error('Failed to show game details:', error)
                );
            });
        });
    }

    createGameCard(game) {
        const coverImage = game['cover_url']
            ? `<img src="${game['cover_url']}" alt="${game.name} cover" loading="lazy">`
            : `<i class="fas fa-gamepad game-cover-placeholder"></i>`;

        // Fixed: Using bracket notation to avoid HTMLTableElement.summary conflict
        const summary = game['summary']
            ? (game['summary'].length > 150 ? game['summary'].slice(0, 150) + '...' : game['summary'])
            : 'No description available.';

        let genres = '';
        try {
            if (game['genres']) {
                const parsedGenres = JSON.parse(game['genres']);
                if (Array.isArray(parsedGenres)) {
                    genres = parsedGenres.slice(0, 2).map(g => g.name).join(', ');
                }
            }
        } catch {
            genres = '';
        }

        const rating = game['rating'] ? Math.round(game['rating']) : null;

        const actionButton = Boolean(game['is_installed'])
            ? `<button class="btn-secondary btn-uninstall" data-game-id="${game.id}">
                 <i class="fas fa-trash"></i> Uninstall
               </button>`
            : `<button class="btn-primary btn-install" data-game-id="${game.id}">
                 <i class="fas fa-download"></i> Install
               </button>`;

        return `
            <div class="game-card" data-game-id="${game.id}">
                <div class="game-cover">
                    ${coverImage}
                </div>
                <div class="game-info">
                    <h3 class="game-title">${game.name}</h3>
                    <p class="game-summary">${summary}</p>
                    <div class="game-meta">
                        ${genres ? `<span class="game-genre">${genres}</span>` : ''}
                        ${rating ? `<span class="game-rating">${rating}%</span>` : ''}
                    </div>
                    <div class="game-actions">
                        ${actionButton}
                        <button class="btn-secondary btn-details" data-game-id="${game.id}">
                            <i class="fas fa-info-circle"></i> Details
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    renderStoreResults(games) {
        const container = document.getElementById('storeResults');
        if (!container) return;

        if (!games || games.length === 0) {
            container.innerHTML = '<p class="no-results">No games found. Try a different search term.</p>';
            return;
        }

        container.innerHTML = games.map(game => this.createStoreGameCard(game)).join('');

        container.querySelectorAll('.btn-add-game').forEach(btn => {
            btn.addEventListener('click', (event) => {
                event.stopPropagation();
                this.addGameFromStore(btn.dataset.igdbId || '', btn.dataset.gameName || '').catch(error =>
                    console.error('Failed to add game from store:', error)
                );
            });
        });
    }

    createStoreGameCard(game) {
        const coverImage = game['cover'] && game['cover'].url
            ? `<img src="https:${game['cover'].url.replace('t_thumb', 't_cover_big')}" alt="${game.name} cover" loading="lazy">`
            : `<i class="fas fa-gamepad game-cover-placeholder"></i>`;

        // Fixed: Using bracket notation to avoid HTMLTableElement.summary conflict
        const summary = game['summary']
            ? (game['summary'].length > 120 ? game['summary'].slice(0, 120) + '...' : game['summary'])
            : 'No description available.';

        const rating = game['rating'] ? Math.round(game['rating']) : null;

        return `
            <div class="game-card store-game-card">
                <div class="game-cover">
                    ${coverImage}
                </div>
                <div class="game-info">
                    <h3 class="game-title">${game.name}</h3>
                    <p class="game-summary">${summary}</p>
                    <div class="game-meta">
                        ${rating ? `<span class="game-rating">${rating}%</span>` : ''}
                    </div>
                    <div class="game-actions">
                        <button class="btn-primary btn-add-game" data-igdb-id="${game.id}" data-game-name="${game.name}">
                            <i class="fas fa-plus"></i> Add to Library
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    async addGameFromStore(igdbId, gameName) {
        const gameData = {
            name: gameName,
            igdb_id: parseInt(igdbId)
        };

        try {
            const response = await this.apiCall('/games', {
                method: 'POST',
                body: JSON.stringify(gameData)
            });

            if (response.success) {
                this.showNotification(`${gameName} added to library!`, 'success');
                await this.fetchGameMetadata(response.data.id);
                if (this.currentSection === 'library') {
                    await this.loadGames();
                }
            }
        } catch (error) {
            this.showNotification(`Failed to add ${gameName}`, 'error');
        }
    }

    async showGameDetails(gameId) {
        try {
            const response = await this.apiCall(`/games/${gameId}`);
            if (response.success) {
                this.renderGameModal(response.data);
                const gameModal = document.getElementById('gameModal');
                if (gameModal) {
                    gameModal.classList.add('active');
                }
            }
        } catch (error) {
            this.showNotification('Failed to load game details', 'error');
        }
    }

    renderGameModal(game) {
        const title = document.getElementById('modalGameTitle');
        const body = document.getElementById('modalBody');
        if (!title || !body) return;

        title.textContent = game.name || 'Unknown';

        const coverImage = game['cover_url']
            ? `<img src="${game['cover_url']}" alt="${game.name} cover" style="max-width: 300px; border-radius: 8px;">`
            : `<div style="width: 300px; height: 400px; background: var(--bg-tertiary); display: flex; align-items: center; justify-content: center; border-radius: 8px;">
                 <i class="fas fa-gamepad" style="font-size: 3rem; color: var(--text-muted);"></i>
               </div>`;

        let genres = 'Unknown';
        let platforms = 'Unknown';
        try {
            if (game['genres']) {
                const parsedGenres = JSON.parse(game['genres']);
                if (Array.isArray(parsedGenres)) {
                    genres = parsedGenres.map(g => g.name).join(', ');
                }
            }
        } catch {
            genres = 'Unknown';
        }

        try {
            if (game['platforms']) {
                const parsedPlatforms = JSON.parse(game['platforms']);
                if (Array.isArray(parsedPlatforms)) {
                    platforms = parsedPlatforms.map(p => p.name).join(', ');
                }
            }
        } catch {
            platforms = 'Unknown';
        }

        const releaseDate = game['release_date'] ? new Date(game['release_date']).getFullYear() : 'Unknown';
        const rating = game['rating'] ? Math.round(game['rating']) : null;
        const isInstalled = Boolean(game['is_installed']);

        // Fixed: Using bracket notation to avoid HTMLTableElement.summary conflict
        const summary = game['summary'] || '';
        const storyline = game['storyline'] || '';

        body.innerHTML = `
            <div style="display: flex; gap: 2rem; margin-bottom: 2rem;">
                <div style="flex-shrink: 0;">
                    ${coverImage}
                </div>
                <div style="flex: 1;">
                    <div style="margin-bottom: 1rem;">
                        <strong>Developer:</strong> ${game['developer'] || 'Unknown'}
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <strong>Publisher:</strong> ${game['publisher'] || 'Unknown'}
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <strong>Release Year:</strong> ${releaseDate}
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <strong>Genres:</strong> ${genres}
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <strong>Platforms:</strong> ${platforms}
                    </div>
                    ${rating ? `<div style="margin-bottom: 1rem;"><strong>Rating:</strong> ${rating}%</div>` : ''}
                    <div style="margin-bottom: 1rem;">
                        <strong>Status:</strong> ${isInstalled ? 'Installed' : 'Not Installed'}
                    </div>
                </div>
            </div>
            ${summary ? `<div style="margin-bottom: 1rem;"><strong>Summary:</strong><p>${summary}</p></div>` : ''}
            ${storyline ? `<div style="margin-bottom: 1rem;"><strong>Storyline:</strong><p>${storyline}</p></div>` : ''}
            <div style="display: flex; gap: 1rem; margin-top: 2rem;">
                ${isInstalled
            ? `<button class="btn-secondary btn-modal-uninstall" data-game-id="${game.id}">
                         <i class="fas fa-trash"></i> Uninstall
                       </button>`
            : `<button class="btn-primary btn-modal-install" data-game-id="${game.id}">
                         <i class="fas fa-download"></i> Install
                       </button>`
        }
                <button class="btn-secondary btn-modal-metadata" data-game-id="${game.id}">
                    <i class="fas fa-sync-alt"></i> Update Metadata
                </button>
            </div>
        `;

        // Add event listeners for modal buttons
        const modalInstallBtn = body.querySelector('.btn-modal-install');
        if (modalInstallBtn) {
            modalInstallBtn.addEventListener('click', (event) => {
                event.stopPropagation();
                this.installGame(game.id).catch(console.error);
            });
        }

        const modalUninstallBtn = body.querySelector('.btn-modal-uninstall');
        if (modalUninstallBtn) {
            modalUninstallBtn.addEventListener('click', (event) => {
                event.stopPropagation();
                this.uninstallGame(game.id).catch(console.error);
            });
        }

        const modalMetadataBtn = body.querySelector('.btn-modal-metadata');
        if (modalMetadataBtn) {
            modalMetadataBtn.addEventListener('click', (event) => {
                event.stopPropagation();
                this.fetchGameMetadata(game.id).catch(console.error);
            });
        }
    }

    showAddGameModal() {
        const modal = document.getElementById('addGameModal');
        if (modal) {
            modal.classList.add('active');
        }
        const input = document.getElementById('gameName');
        if (input) {
            input.focus();
        }
    }

    hideAddGameModal() {
        const modal = document.getElementById('addGameModal');
        if (modal) {
            modal.classList.remove('active');
        }
        const form = document.getElementById('addGameForm');
        if (form) {
            form.reset();
        }
    }

    updateStats() {
        const totalGames = this.games.length;

        const totalGamesElem = document.getElementById('totalGames');
        if (totalGamesElem) {
            totalGamesElem.textContent = String(totalGames);
        }

        const totalSizeBytes = this.games.reduce((total, game) => {
            return total + (game['file_size'] || 0);
        }, 0);

        const totalSizeElem = document.getElementById('totalSize');
        if (totalSizeElem) {
            totalSizeElem.textContent = this.formatFileSize(totalSizeBytes);
        }
    }

    updateCategoryCounts() {
        const total = this.games.length;
        const installed = this.games.filter(game => Boolean(game['is_installed'])).length;
        const uninstalled = total - installed;

        const allCountElem = document.getElementById('allCount');
        if (allCountElem) {
            allCountElem.textContent = String(total);
        }

        const installedCountElem = document.getElementById('installedCount');
        if (installedCountElem) {
            installedCountElem.textContent = String(installed);
        }

        const uninstalledCountElem = document.getElementById('uninstalledCount');
        if (uninstalledCountElem) {
            uninstalledCountElem.textContent = String(uninstalled);
        }
    }

    formatFileSize(bytes) {
        if (bytes === 0) return '0 GB';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    showLoading(show) {
        const loading = document.getElementById('loading');
        const container = document.getElementById('gamesContainer');
        if (!loading || !container) return;

        if (show) {
            loading.style.display = 'flex';
            container.style.display = 'none';
        } else {
            loading.style.display = 'none';
            container.style.display = 'grid';
        }
    }

    showStoreLoading(show) {
        const container = document.getElementById('storeResults');
        if (!container) return;

        if (show) {
            container.innerHTML = `
                <div class="loading">
                    <div class="spinner"></div>
                    <span>Searching games...</span>
                </div>
            `;
        }
    }

    async refreshLibrary() {
        this.showNotification('Refreshing library...', 'info');
        await this.loadGames();
    }

    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.innerHTML = `
            <i class="fas fa-${this.getNotificationIcon(type)}"></i>
            <span>${message}</span>
            <button class="notification-close">
                <i class="fas fa-times"></i>
            </button>
        `;

        document.body.appendChild(notification);

        setTimeout(() => {
            notification.remove();
        }, 5000);

        const closeButton = notification.querySelector('.notification-close');
        if (closeButton) {
            closeButton.addEventListener('click', () => {
                notification.remove();
            });
        }
    }

    getNotificationIcon(type) {
        const icons = {
            success: 'check-circle',
            error: 'exclamation-circle',
            warning: 'exclamation-triangle',
            info: 'info-circle'
        };
        return icons[type] || 'info-circle';
    }
}

window.addEventListener('DOMContentLoaded', () => {
    window.gameHub = new GameHub();
});
