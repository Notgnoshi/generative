import itertools
from typing import Dict, Iterable

# TODO: Implement context-free, stochastic, context sensitive, and parametric grammars
class ContextFreeGrammar:
    """Apply L-System production rules to strings."""

    def __init__(self, rules: Dict[str, str]):
        """Initialize a Grammar with a set of production rules.

        :param rules: A dict of (symbol, rewrite) production rules.
        """
        self.rules = rules

    def _apply(self, text: Iterable[str]) -> Iterable[str]:
        """Apply the production rules to the given text, once."""
        rewrites = (self.rules.get(token, token) for token in text)
        return itertools.chain.from_iterable(rewrites)

    def apply(self, text: str, times: int = 1) -> Iterable[str]:
        """Apply the production rules to the given text.

        :param text: The text to apply the production rules to.
        :param times: The number of times to sequentially apply the rules.
        """
        for _ in range(times):
            # It's necessary to cache the intermediate results like this.
            text = self._apply(text)
        return text
