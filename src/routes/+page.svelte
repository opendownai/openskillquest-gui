<script>
  import { invoke } from "@tauri-apps/api/core";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { onMount } from "svelte";

  let game = $state({ level: 1, exp: 0, max_exp: 500, installed: [], daily_quests: 0, player_name: "Agent Hunter" });
  let skills = $state([]);
  let selectedSkill = $state(null);
  let loading = $state(true);
  let showCopied = $state(false);
  let levelUp = $state(false);
  let levelDisplay = $state(1);
  let newTitle = $state("");

  const rarityColors = {
    Common: "#9ca3af",
    Uncommon: "#22c55e",
    Rare: "#06b6d4",
    Epic: "#a855f7",
    Legendary: "#eab308"
  };

  const titles = [
    { minLevel: 1, title: "Novice Hunter" },
    { minLevel: 5, title: "Skill Seeker" },
    { minLevel: 10, title: "Agent Apprentice" },
    { minLevel: 20, title: "Skill Collector" },
    { minLevel: 30, title: "Quest Master" },
    { minLevel: 50, title: "Skill Master" },
    { minLevel: 75, title: "Agent Legend" },
    { minLevel: 100, title: "OpenDown Hero" }
  ];

  function getTitle(level) {
    for (let i = titles.length - 1; i >= 0; i--) {
      if (level >= titles[i].minLevel) {
        return titles[i].title;
      }
    }
    return "Novice Hunter";
  }

  onMount(async () => {
    await refreshData();
  });

  async function refreshData() {
    loading = true;
    try {
      game = await invoke("get_game_state");
      skills = await invoke("fetch_skills");
      levelDisplay = game.level;
      if (!game.player_name) {
        game.player_name = "Agent Hunter";
      }
    } catch (e) {
      console.error(e);
    }
    loading = false;
  }

  async function selectSkill(skill) {
    try {
      const info = await invoke("get_skill_info", { skillName: skill.name || skill.slug });
      selectedSkill = info;
    } catch (e) {
      console.error(e);
    }
  }

  async function completeQuest() {
    if (!selectedSkill) return;
    const prevLevel = game.level;
    try {
      game = await invoke("complete_quest", { skillName: selectedSkill.name });
      if (game.level > prevLevel) {
        levelUp = true;
        levelDisplay = game.level;
        newTitle = getTitle(game.level);
        setTimeout(() => {
          levelUp = false;
          newTitle = "";
        }, 4000);
      }
      selectedSkill = null;
    } catch (e) {
      console.error(e);
    }
  }

  async function copyInstallCmd() {
    if (selectedSkill?.install_cmd) {
      await writeText(selectedSkill.install_cmd);
      showCopied = true;
      setTimeout(() => showCopied = false, 2000);
    }
  }

  async function share() {
    const text = await invoke("get_share_text", { playerName: game.player_name || "Agent Hunter" });
    await writeText(text);
  }

  function getRarityColor(rarity) {
    return rarityColors[rarity] || "#9ca3af";
  }

  function isInstalled(skill) {
    return game.installed.includes(skill.name) || game.installed.includes(skill.slug);
  }

  function getSkillInfo(skillName) {
    return skills.find(s => s.name === skillName || s.slug === skillName);
  }

  async function viewInstalledSkill(skillName) {
    const skill = getSkillInfo(skillName);
    if (skill) {
      await selectSkill(skill);
    }
  }
</script>

<main class="app">
  <div class="header">
    <div class="logo">
      <img src="https://cdn.opendown.ai/opendown-ai-2.png" alt="Logo" class="logo-img" />
      <span class="logo-text">OpenSkillQuest</span>
    </div>
    <div class="header-actions">
      <span class="player-name">{game.player_name || "Agent Hunter"}</span>
      <button class="refresh-btn" onclick={refreshData} disabled={loading}>
        {loading ? "..." : "↻"}
      </button>
    </div>
  </div>

  <div class="content">
    <div class="left-panel">
      <div class="character-card">
        <div class="avatar">
          <span class="avatar-icon">🗡️</span>
        </div>
        <div class="character-info">
          <h2>Level {game.level}</h2>
          <p class="title">{getTitle(game.level)}</p>
          <div class="exp-bar">
            <div class="exp-fill" style="width: {(game.exp / game.max_exp) * 100}%"></div>
          </div>
          <p class="exp-text">{game.exp} / {game.max_exp} EXP</p>
        </div>
      </div>

      <div class="stats">
        <div class="stat">
          <span class="stat-label">Daily Quests</span>
          <span class="stat-value">{game.daily_quests}/3</span>
        </div>
        <div class="stat">
          <span class="stat-label">Skills Installed</span>
          <span class="stat-value">{game.installed.length}</span>
        </div>
      </div>

      {#if game.installed.length > 0}
        <div class="build-list">
          <h3>Your Build</h3>
          <div class="skills-chips">
            {#each game.installed as skill}
              <button class="chip installed" onclick={() => viewInstalledSkill(skill)}>
                {skill}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <button class="share-btn" onclick={share}>
        📤 Share Build
      </button>
    </div>

    <div class="right-panel">
      <div class="quests-header">
        <h3 class="panel-title">Available Quests</h3>
        <button class="refresh-quests-btn" onclick={refreshData} disabled={loading}>
          {loading ? "..." : "↻ Refresh"}
        </button>
      </div>
      <div class="quests-grid">
        {#each skills as skill}
          <button 
            class="quest-card" 
            class:installed={isInstalled(skill)}
            onclick={() => selectSkill(skill)}
          >
            <div class="quest-header">
              <span class="quest-name">{skill.name || skill.slug}</span>
              <span class="quest-rarity" style="color: {getRarityColor(skill.rarity || 'Common')}">
                {skill.rarity || 'Common'}
              </span>
            </div>
            <p class="quest-desc">{(skill.desc || "").slice(0, 60)}...</p>
            <div class="quest-meta">
              <span>⬇ {skill.downloads?.toLocaleString() || 0}</span>
              <span>⭐ {skill.stars || 0}</span>
              <span class="quest-exp">+{skill.exp || 10} EXP</span>
            </div>
            {#if isInstalled(skill)}
              <div class="installed-badge">✅ Installed</div>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#if selectedSkill}
    <div class="modal-overlay" role="button" tabindex="0" onclick={() => selectedSkill = null} onkeydown={(e) => e.key === 'Escape' && (selectedSkill = null)}>
      <div class="modal" role="dialog" onclick={(e) => e.stopPropagation()}>
        <h2>{selectedSkill.name}</h2>
        <p class="modal-desc">{selectedSkill.desc}</p>
        
        <div class="modal-stats">
          <span style="color: {getRarityColor(selectedSkill.rarity)}">
            {selectedSkill.rarity}
          </span>
          <span>⬇ {selectedSkill.downloads?.toLocaleString()}</span>
          <span>⭐ {selectedSkill.stars}</span>
        </div>

        <div class="install-cmd">
          <code>{selectedSkill.install_cmd}</code>
          <button class="copy-btn" onclick={copyInstallCmd}>
            {showCopied ? "✓ Copied!" : "📋 Copy"}
          </button>
        </div>

        <div class="modal-actions">
          {#if !isInstalled(selectedSkill)}
            <button class="accept-btn" onclick={completeQuest}>
              ✓ Mark Complete (+{selectedSkill.exp} EXP)
            </button>
          {:else}
            <button class="complete-btn" onclick={() => selectedSkill = null}>
              Close
            </button>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  {#if levelUp}
    <div class="levelup-overlay" role="button" tabindex="0" onclick={() => levelUp = false} onkeydown={(e) => e.key === 'Escape' && (levelUp = false)}>
      <div class="levelup-animation">
        <span class="levelup-text">🎉 LEVEL UP!</span>
        <span class="levelup-number">Level {levelDisplay}</span>
        {#if newTitle}
          <span class="new-title">New Title: {newTitle}</span>
        {/if}
      </div>
    </div>
  {/if}
</main>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    background: #0a0a0f;
    color: #fff;
    font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
    overflow: hidden;
  }

  .app {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: 
      linear-gradient(rgba(0, 245, 255, 0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(0, 245, 255, 0.03) 1px, transparent 1px),
      #0a0a0f;
    background-size: 50px 50px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 24px;
    border-bottom: 1px solid rgba(0, 245, 255, 0.1);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .logo-img {
    width: 40px;
    height: 40px;
    border-radius: 8px;
    object-fit: contain;
    background: rgba(255, 255, 255, 0.05);
    padding: 4px;
  }

  .logo-text {
    font-size: 20px;
    font-weight: 700;
    background: linear-gradient(90deg, #00f5ff, #8b5cf6, #06b6d4);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .player-name {
    font-size: 14px;
    color: #a1a1aa;
    background: rgba(139, 92, 246, 0.1);
    padding: 6px 12px;
    border-radius: 8px;
    border: 1px solid rgba(139, 92, 246, 0.2);
  }

  .refresh-btn {
    width: 40px;
    height: 40px;
    border-radius: 8px;
    border: 1px solid rgba(0, 245, 255, 0.2);
    background: rgba(0, 245, 255, 0.05);
    color: #00f5ff;
    font-size: 18px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .refresh-btn:hover {
    background: rgba(0, 245, 255, 0.1);
    border-color: #00f5ff;
  }

  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
    padding: 20px;
    gap: 20px;
  }

  .left-panel {
    width: 320px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .character-card {
    background: linear-gradient(135deg, rgba(20, 20, 30, 0.9), rgba(30, 30, 45, 0.9));
    border: 1px solid rgba(139, 92, 246, 0.3);
    border-radius: 16px;
    padding: 20px;
    display: flex;
    gap: 16px;
  }

  .avatar {
    width: 64px;
    height: 64px;
    border-radius: 12px;
    background: linear-gradient(135deg, #8b5cf6, #06b6d4);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 32px;
  }

  .character-info {
    flex: 1;
  }

  .character-info h2 {
    font-size: 24px;
    background: linear-gradient(90deg, #00f5ff, #8b5cf6);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .title {
    font-size: 12px;
    color: #8b5cf6;
    margin-bottom: 8px;
  }

  .exp-bar {
    height: 8px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    margin: 8px 0;
    overflow: hidden;
  }

  .exp-fill {
    height: 100%;
    background: linear-gradient(90deg, #00f5ff, #8b5cf6);
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .exp-text {
    font-size: 12px;
    color: #a1a1aa;
  }

  .stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .stat {
    background: rgba(20, 20, 30, 0.8);
    border: 1px solid rgba(0, 245, 255, 0.1);
    border-radius: 12px;
    padding: 12px;
    text-align: center;
  }

  .stat-label {
    display: block;
    font-size: 11px;
    color: #71717a;
    margin-bottom: 4px;
  }

  .stat-value {
    font-size: 20px;
    font-weight: 600;
    color: #00f5ff;
  }

  .build-list {
    background: rgba(20, 20, 30, 0.8);
    border: 1px solid rgba(0, 245, 255, 0.1);
    border-radius: 12px;
    padding: 12px;
    flex: 1;
    overflow: hidden;
  }

  .build-list h3 {
    font-size: 14px;
    color: #a1a1aa;
    margin-bottom: 8px;
  }

  .skills-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .chip {
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 6px;
    background: rgba(139, 92, 246, 0.2);
    color: #a855f7;
    border: 1px solid rgba(139, 92, 246, 0.3);
    cursor: pointer;
    transition: all 0.2s;
  }

  .chip:hover {
    background: rgba(139, 92, 246, 0.3);
    transform: scale(1.05);
  }

  .chip.installed {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
    border-color: rgba(34, 197, 94, 0.3);
  }

  .share-btn {
    padding: 14px;
    border-radius: 12px;
    border: 1px solid rgba(139, 92, 246, 0.3);
    background: linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(6, 182, 212, 0.2));
    color: #fff;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .share-btn:hover {
    background: linear-gradient(135deg, rgba(139, 92, 246, 0.3), rgba(6, 182, 212, 0.3));
    border-color: #8b5cf6;
  }

  .right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .quests-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .panel-title {
    font-size: 16px;
    color: #a1a1aa;
  }

  .refresh-quests-btn {
    padding: 6px 12px;
    border-radius: 6px;
    border: 1px solid rgba(0, 245, 255, 0.2);
    background: rgba(0, 245, 255, 0.05);
    color: #00f5ff;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .refresh-quests-btn:hover {
    background: rgba(0, 245, 255, 0.1);
  }

  .quests-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
    overflow-y: auto;
    padding-right: 8px;
  }

  .quests-grid::-webkit-scrollbar {
    width: 6px;
  }

  .quests-grid::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 3px;
  }

  .quests-grid::-webkit-scrollbar-thumb {
    background: rgba(0, 245, 255, 0.2);
    border-radius: 3px;
  }

  .quest-card {
    background: rgba(20, 20, 30, 0.8);
    border: 1px solid rgba(0, 245, 255, 0.1);
    border-radius: 12px;
    padding: 14px;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s;
    position: relative;
  }

  .quest-card:hover {
    border-color: rgba(0, 245, 255, 0.3);
    background: rgba(30, 30, 45, 0.9);
    transform: scale(1.02);
    box-shadow: 0 0 20px rgba(0, 245, 255, 0.15);
  }

  .quest-card.installed {
    border-color: rgba(34, 197, 94, 0.3);
  }

  .quest-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .quest-name {
    font-weight: 600;
    font-size: 14px;
    color: #fff;
  }

  .quest-rarity {
    font-size: 10px;
    font-weight: 600;
  }

  .quest-desc {
    font-size: 12px;
    color: #71717a;
    margin-bottom: 10px;
    line-height: 1.4;
  }

  .quest-meta {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: #a1a1aa;
  }

  .quest-exp {
    color: #22c55e;
    font-weight: 600;
    margin-left: auto;
  }

  .installed-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    font-size: 10px;
    color: #22c55e;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    cursor: pointer;
  }

  .modal {
    background: linear-gradient(135deg, #1a1a25, #12121a);
    border: 1px solid rgba(139, 92, 246, 0.3);
    border-radius: 20px;
    padding: 24px;
    width: 420px;
    max-width: 90vw;
    cursor: default;
  }

  .modal h2 {
    font-size: 22px;
    margin-bottom: 12px;
    background: linear-gradient(90deg, #00f5ff, #8b5cf6);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .modal-desc {
    color: #a1a1aa;
    font-size: 14px;
    line-height: 1.5;
    margin-bottom: 16px;
  }

  .modal-stats {
    display: flex;
    gap: 16px;
    margin-bottom: 16px;
    font-size: 13px;
  }

  .install-cmd {
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(0, 245, 255, 0.2);
    border-radius: 10px;
    padding: 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
  }

  .install-cmd code {
    flex: 1;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    color: #22c55e;
  }

  .copy-btn {
    padding: 6px 12px;
    border-radius: 6px;
    border: 1px solid rgba(0, 245, 255, 0.3);
    background: rgba(0, 245, 255, 0.1);
    color: #00f5ff;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .copy-btn:hover {
    background: rgba(0, 245, 255, 0.2);
  }

  .modal-actions {
    display: flex;
    gap: 12px;
  }

  .accept-btn {
    flex: 1;
    padding: 14px;
    border-radius: 10px;
    border: none;
    background: linear-gradient(135deg, #8b5cf6, #06b6d4);
    color: #fff;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .accept-btn:hover {
    opacity: 0.9;
    transform: scale(1.02);
  }

  .complete-btn {
    flex: 1;
    padding: 14px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: transparent;
    color: #a1a1aa;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .complete-btn:hover {
    border-color: rgba(255, 255, 255, 0.4);
    color: #fff;
  }

  .levelup-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.9);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
    cursor: pointer;
    animation: fadeIn 0.3s ease;
  }

  .levelup-animation {
    text-align: center;
    animation: scaleIn 0.5s ease;
  }

  .levelup-text {
    display: block;
    font-size: 48px;
    font-weight: 700;
    background: linear-gradient(90deg, #eab308, #f97316, #22c55e);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    animation: pulse 1s ease infinite;
  }

  .levelup-number {
    display: block;
    font-size: 64px;
    font-weight: 700;
    color: #fff;
    margin-top: 16px;
  }

  .new-title {
    display: block;
    font-size: 20px;
    color: #8b5cf6;
    margin-top: 12px;
    padding: 8px 16px;
    background: rgba(139, 92, 246, 0.2);
    border-radius: 8px;
    border: 1px solid rgba(139, 92, 246, 0.4);
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes scaleIn {
    from { transform: scale(0.5); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.05); }
  }
</style>
