# Spotify Dropdown — v0.2.2 Alpha

Controlador local do Spotify Desktop para Windows 11 x64, feito em Rust/egui, sem OAuth. Este executável é uma **versão candidata da mesma v0.2.2-alpha**, pendente de validação manual. A publicação da Release aguarda aprovação explícita do usuário.

## Abrir e usar

1. Extraia o ZIP `SpotifyDropdown-v0.2.2-alpha-windows-x64.zip` em uma pasta gravável e abra **`Spotify Dropdown.exe`**. A candidata atual fica em `dist\v0.2.2-alpha\candidata\Spotify Dropdown.exe`; o executável anterior foi preservado. Não há instalador nesta atualização.
2. Abra o Spotify Desktop e reproduza uma música neste computador. Mantenha ativado **Configurações → Exibição → Mostrar o Desktop Overlay quando os controles de reprodução estiverem sendo usados**.
3. Use **•••** para abrir os ajustes na própria janela; **X** retorna ao controlador. O botão direito não abre mais o menu. A aba **Sobre** informa a versão.
4. Ative o **modo dropdown**, escolha monitor e alinhamento. Ao recolher, o painel fica invisível; encoste o cursor no topo, na região correspondente ao painel, para abri-lo. O slider define o atraso de recolhimento, com padrão de **0,8 s**.
5. Redimensione pelos cantos. No dropdown, use os **cantos inferiores**: a borda superior permanece fixa; no alinhamento central, a largura varia para os dois lados. Ao reduzir a altura, o modo compacto organiza **capa e informações à esquerda, controles à direita e progresso abaixo**. A capa continua visível, com tamanho proporcional. Arrastar o cabeçalho transforma o painel em janela livre.
6. **Esconder**, no menu, mantém o programa em execução na bandeja e desativa a abertura pelo topo. **Dois cliques no ícone da bandeja** reabrem a janela. **Fechar** encerra o aplicativo. O ícone pode estar entre os ícones ocultos da bandeja.

Metadados e anterior/reproduzir-pausar/próxima usam a sessão de mídia do Windows. O progresso é somente leitura. Passe o cursor sobre volume para abrir o slider vertical, sem porcentagem numérica; **clique no alto-falante para mutar ou desmutar**. O controle atua nas sessões de áudio do Spotify no mixer do Windows, separadamente do slider interno do Spotify, e não altera o volume global. O mute preserva o ganho de cada sessão. Sem sessão Spotify controlável, o botão fica indisponível. Adicionar à biblioteca e playlists continuam desabilitados.

Os controles visuais foram reduzidos mantendo áreas clicáveis confortáveis. O layout limita a expansão dos grupos em janelas largas e reduz espaços vazios. Menu, botões de ação e seleções usam detalhes verdes; sliders adotam preenchimento verde e cursor branco circular. A antiga opção de manter a janela no topo foi removida somente do menu.

As preferências ficam em `settings.ini`, ao lado do executável. Os arquivos das versões anteriores foram preservados. Para reutilizar seus ajustes na pasta nova, copie o `settings.ini` anterior antes de abrir a nova versão; tamanhos compatíveis são mantidos e valores antigos fora dos limites são ajustados.

## Validação e limites

Consulte [MANUAL-TESTS.md](MANUAL-TESTS.md) para verificar janela, bandeja, redimensionamento, menu, textos, mídia e volume. **Compilar com sucesso não aprova esses comportamentos.** Nesta versão não se abre o aplicativo nem se automatiza a área de trabalho durante o desenvolvimento.

Tela cheia exclusiva, jogos que prendem o cursor e outras janelas que disputam prioridade impedem garantia universal de sobreposição. DPI/múltiplos monitores, roteamento de áudio, mute e textos longos também precisam de conferência manual. Se houver várias sessões Spotify, o ícone representa uma sessão de referência e a ação de mute é aplicada às sessões Spotify controláveis; mudanças externas podem levar até a próxima atualização para aparecer.

## Compilar novamente

As ferramentas deste ambiente ficam em `.tools`, sem alteração permanente do PATH. Os scripts de preparação dependem de downloads previamente autorizados; `setup-rust.ps1` pressupõe o instalador oficial e o LLVM-MinGW já disponíveis. Depois de preparar as ferramentas e obter as dependências com `scripts\dev.ps1 fetch`:

```powershell
pwsh -NoProfile -File .\scripts\dev.ps1 check
pwsh -NoProfile -File .\scripts\dev.ps1 release
pwsh -NoProfile -File .\scripts\package.ps1 -Candidate
```

O pacote contém apenas executável e documentação. `SHA256SUMS.txt`, ao lado dele, identifica o executável e o ZIP. Não execute `build-installer.ps1` nesta atualização menor. Os scripts históricos de teste/instalador foram preservados, mas não integram a validação da v0.2.2.

Código: `player.rs`, `layout.rs` e `ui.rs` cuidam da interface; `window.rs`, `dropdown.rs` e `preferences.rs` cuidam da janela; `media.rs`, `model.rs` e `volume.rs` integram o Windows. O ponto de recuperação anterior está na tag local `snapshot-v0.2.1`.

Repositório oficial: [Velsurr/spotify-dropdown](https://github.com/Velsurr/spotify-dropdown). As fontes podem ser sincronizadas durante a preparação, mas **tag definitiva e Release não devem ser publicadas antes da aprovação manual**. Versões usam SemVer, incluindo o sufixo `-alpha`; consulte [CHANGELOG.md](CHANGELOG.md) e [RELEASE-NOTES.md](RELEASE-NOTES.md). Ferramentas, executáveis, preferências e diagnósticos locais ficam fora do Git. Não afiliado ao Spotify.
