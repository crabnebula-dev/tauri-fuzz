import Reveal from "reveal.js";
import Markdown from "reveal.js/plugin/markdown/markdown.esm.js";
import RevealZoom from "reveal.js/plugin/zoom/zoom.esm.js"
import RevealNotes from "reveal.js/plugin/notes/notes.esm.js"
import RevealSearch from "reveal.js/plugin/search/search.esm.js"
import RevealHighlight from "reveal.js/plugin/highlight/highlight.esm.js"

let deck = new Reveal({
				controls: true,
				progress: true,
				center: true,
				hash: true,
				keyboard: true,

				// Learn about plugins: https://revealjs.com/plugins/
				plugins: [ RevealZoom, RevealNotes, RevealSearch, RevealHighlight, Markdown ]
});


deck.initialize();

		// <script src="dist/reveal.js"></script>
		// <script src="plugin/zoom/zoom.js"></script>
		// <script src="plugin/notes/notes.js"></script>
		// <script src="plugin/search/search.js"></script>
		// <script src="plugin/markdown/markdown.js"></script>
		// <script src="plugin/highlight/highlight.js"></script>
		// <script>

		// 	// Also available as an ES module, see:
		// 	// https://revealjs.com/initialization/
		// 	Reveal.initialize({
		// 		controls: true,
		// 		progress: true,
		// 		center: true,
		// 		hash: true,
		// 		keyboard: true,
		//
		// 		// Learn about plugins: https://revealjs.com/plugins/
		// 		plugins: [ RevealZoom, RevealNotes, RevealSearch, RevealMarkdown, RevealHighlight ]
		// 	});
		//
		// </script>

