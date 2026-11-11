# Spotify Dropdown — 0.2.1

Controlador local do Spotify Desktop para Windows 11, feito em Rust/egui, sem OAuth.

## Abrir e testar

- **Portátil:** abra `dist\v0.2.1\Spotify Dropdown.exe`.
- **Instalador:** abra `dist\v0.2.1\SpotifyDropdown-Setup-0.2.1.exe`. Instala para o usuário atual, sem administrador, e cria um atalho no menu Iniciar. A pasta padrão é `%LOCALAPPDATA%\Programs\SpotifyDropdown`.
- No Spotify Desktop, reproduza uma música neste computador e mantenha ativado **Configurações → Exibição → Mostrar o Desktop Overlay quando os controles de reprodução estiverem sendo usados**.
- Abra **••• → Ajustes**, ou clique com o botão direito na janela. Marque **Ativar modo dropdown**, escolha monitor e alinhamento. O painel fica totalmente invisível ao recolher. Encoste o cursor no topo, na região correspondente à largura/alinhamento escolhidos, para abri-lo.
- O atraso de recolhimento é ajustável no slider, com padrão de **0,8 s**. Menus abertos e arrastes mantêm a janela visível. Não há atalhos específicos do aplicativo nesta versão.
- Arraste o cabeçalho para mover a janela; isso retira o painel do modo dropdown. No dropdown, redimensione apenas pelo **canto inferior direito**. No modo livre, use qualquer borda ou canto. Tamanhos também podem ser ajustados no menu. A aba **Sobre** mostra a versão.

**Funcionam:** capa, faixa, artista, álbum, anterior, reproduzir/pausar e próxima, pela sessão de mídia do Windows. O progresso é somente leitura e depende dos tempos fornecidos pelo Spotify. Favorito, volume e playlists permanecem indisponíveis e identificados como tal.

As preferências ficam em `settings.ini`, ao lado do executável. O instalador preserva esse arquivo nas atualizações e na desinstalação. As versões anteriores na pasta do projeto continuam disponíveis.

O observador do cursor funciona sem a janela estar em foco ou visível, e o dropdown reaparece como janela no topo, sem tomar o foco do outro aplicativo. **Tela cheia exclusiva, jogos que prendem o cursor e outras janelas que disputam a prioridade não permitem garantia universal.** Prefira tela cheia sem bordas para usar com jogos. Não há injeção em jogos ou ganchos de teclado.

## Compilar

Rust estável, Cargo, LLVM-MinGW e NSIS portátil ficam em `.tools` neste ambiente. Os scripts não alteram o PATH permanentemente. Para preparar outra máquina, revise e autorize os downloads dos scripts de preparação antes de executá-los. O script `setup-rust.ps1` pressupõe o instalador oficial rustup-init.exe e o LLVM-MinGW já disponíveis em `.tools`; ele não é um instalador completo de todas as ferramentas. Após preparar as ferramentas e buscar as dependências com `scripts\dev.ps1 fetch`:

```powershell
pwsh -NoProfile -File .\scripts\dev.ps1 test
pwsh -NoProfile -File .\scripts\dev.ps1 clippy
pwsh -NoProfile -File .\scripts\dev.ps1 release
pwsh -NoProfile -File .\scripts\package.ps1
pwsh -NoProfile -File .\scripts\build-installer.ps1
```

Código: `player.rs` e `layout.rs` organizam a interface; `window.rs`, `dropdown.rs` e `preferences.rs` cuidam da janela; `media.rs` e `model.rs` integram o Spotify. `fullscreen-fixture` é uma janela de teste local, excluída da distribuição.

Versões usam `MAJOR.MINOR.PATCH`; consulte [CHANGELOG.md](CHANGELOG.md). Ferramentas, executáveis gerados, preferências e diagnósticos locais são ignorados pelo Git. Não afiliado ao Spotify.
