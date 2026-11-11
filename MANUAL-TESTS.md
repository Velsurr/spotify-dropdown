# Validação manual — candidata à v0.2.2 Alpha

**Checklist para o usuário; ainda não executado.** A versão permanece `0.2.2-alpha`. Compilação e análise estática não aprovam aparência ou funcionamento. Não publicar a tag definitiva nem a Release até a aprovação explícita do usuário.

Extraia o ZIP em pasta gravável e abra `Spotify Dropdown.exe`. Compare com suas capturas de referência em janela ampla, estreita e na altura mínima.

- [ ] **Composição normal:** controles visualmente menores, áreas clicáveis confortáveis e menos espaço vazio entre informações e reprodução. Janelas muito largas não devem espalhar excessivamente os grupos.
- [ ] **Compacto horizontal:** na altura mínima, capa sempre visível e proporcional à esquerda, informações ao lado, anterior/reproduzir-pausar/próxima à direita e progresso discreto abaixo. Nenhum controle ou texto deve se sobrepor.
- [ ] **Redimensionamento:** varie largura e altura devagar, inclusive perto da transição de layout. Confirme ausência de cortes, saltos bruscos ou tremulações; o limite mínimo vertical permanece preservado.
- [ ] **Textos:** música, artista e álbum longos devem permanecer em uma linha, com rolagem para a direita somente quando necessário e sem ultrapassar suas áreas.
- [ ] **Menu:** confira ausência da opção de manter a janela no topo; Esconder/Fechar e seleções ativadas em verde, sem aparência de seleção nas opções desativadas. X volta ao controlador; clique fora fecha os ajustes; botão direito não abre menu.
- [ ] **Sliders:** atraso de recolhimento e volume com preenchimento verde, cursor branco circular e trilho cinza-escuro. Confira o indicador de progresso com a mesma identidade visual, mantendo-o somente leitura.
- [ ] **Volume:** passe o cursor no alto-falante, ajuste o slider vertical e mova entre botão/painel. Ele deve permanecer aberto nesse trajeto, fechar ao sair e não mostrar porcentagem numérica nem ultrapassar a janela.
- [ ] **Mute:** clique para mutar, confira a mudança do ícone e clique para restaurar o áudio no nível anterior. Teste também mute/volume alterados pelo Mixer do Windows; aguarde a atualização do estado na interface.
- [ ] **Isolamento do áudio:** ajuste/mute Spotify com outro aplicativo reproduzindo som. O volume global e o áudio do outro aplicativo devem permanecer inalterados. Repita em outras saídas ou sessões, quando disponíveis.
- [ ] **Regressões do dropdown:** confirme topo fixo durante todo o arraste, largura simétrica no alinhamento central, limites do monitor, abertura pelo topo e recolhimento com o atraso escolhido. Repita em outra escala DPI/monitor, quando possível.
- [ ] **Bandeja e comandos:** Esconder mantém o ícone e desativa o gatilho do topo; duplo clique reabre. Fechar encerra o processo e remove o ícone. Abrir/recolher não deve criar botão na barra de tarefas/Alt+Tab.
- [ ] **Mídia e indisponibilidade:** verifique capa, metadados, anterior/reproduzir-pausar/próxima e estado do cabeçalho; feche/reabra o Spotify para conferir desabilitação e reconexão. Adicionar à biblioteca e playlists continuam indisponíveis.

Tela cheia exclusiva, jogos que prendem o cursor e múltiplos dispositivos de áudio exigem conferência específica; não há garantia universal. Ao relatar um problema, indique tamanho da janela, modo, alinhamento, monitor/escala DPI e sequência de ações. Capturas e gravações podem ser feitas manualmente pelo usuário.
