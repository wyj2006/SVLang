from collections.abc import Iterable
from dataclasses import dataclass
from itertools import product
from pprint import *


@dataclass
class Grammar:
    rules: list[Rule]

    def to_pest(self):
        code = ""
        for rule in self.rules:
            code += rule.to_pest()
            code += "\n"
        return code


@dataclass(frozen=True)
class Rule:
    name: str
    item: Item

    def to_pest(self):
        code = f"{self.name} = {{"
        code += self.item.to_pest()
        code += f"}}"
        return code

    def first(self):
        return self.item.first()


class Item:
    def to_pest(self):
        return ""

    def first(self):
        return {self}

    def expand(self):
        return self

    def replace(self, old, new):
        if self == old:
            return new
        return self

    def remove_empty(self):
        return self

    def normalize(self):
        return self.expand().remove_empty()


@dataclass(frozen=True)
class Choice(Item):
    items: tuple[Item]

    def to_pest(self):
        code = self.items[0].to_pest()
        for sequence in self.items[1:]:
            code += f"|{sequence.to_pest()}"
        return code

    def first(self):
        a = set()
        for i in self.items:
            a |= i.first()
        return a

    def __post_init__(self):
        object.__setattr__(self, "items", tuple(self.items))

    def expand(self):
        new_items = []
        for i in self.items:
            match i.expand():
                case Choice(items):
                    new_items.extend(items)
                case a:
                    new_items.append(a)
        if len(new_items) == 0:
            return Empty()
        if len(new_items) == 1:
            return new_items[0]
        return Choice(new_items)

    def replace(self, old, new):
        return Choice([i.replace(old, new) for i in self.items])

    def remove_empty(self):
        new_items = []
        for i in self.items:
            item = i.remove_empty()
            if not isinstance(item, Empty):
                new_items.append(item)
        if not new_items:
            return Empty()
        return Choice(new_items)


@dataclass(frozen=True)
class Sequence(Item):
    items: tuple[Item]

    def to_pest(self):
        if isinstance(self.items[0], Choice):
            code = f"({self.items[0].to_pest()})"
        else:
            code = self.items[0].to_pest()
        for item in self.items[1:]:
            if isinstance(item, Choice):
                code += f"~({item.to_pest()})"
            else:
                code += f"~{item.to_pest()}"
        return code

    def first(self):
        a = set()
        for i in self.items:
            b = i.first()
            if None in b:
                a |= b - {None}
            else:
                a |= b
                break
        else:
            a |= {None}
        return a

    def __post_init__(self):
        object.__setattr__(self, "items", tuple(self.items))

    def expand(self):
        combine = []
        new_items = []
        for i in self.items:
            match i.expand():
                case Sequence(items):
                    new_items.extend(items)
                case Choice(items):
                    combine.append([new_items])
                    combine.append(items)
                    new_items = []
                case a:
                    new_items.append(a)
        if new_items:
            if not combine:
                if len(new_items) == 1:
                    return new_items[0]
                return Sequence(new_items)
            combine.append([new_items])
        if len(combine) == 0:
            return Empty()

        return Choice([Sequence(flatten(i)) for i in product(*combine)]).expand()

    def replace(self, old, new):
        return Sequence([i.replace(old, new) for i in self.items])

    def remove_empty(self):
        new_items = []
        for i in self.items:
            item = i.remove_empty()
            if not isinstance(item, Empty):
                new_items.append(item)
        if not new_items:
            return Empty()
        return Sequence(new_items)


@dataclass(frozen=True)
class Repeat(Item):
    item: Item

    def to_pest(self):
        if isinstance(self.item, (Choice, Sequence)):
            return f"({self.item.to_pest()})*"
        else:
            return f"{self.item.to_pest()}*"

    def first(self):
        return self.item.first() | {None}

    def expand(self):
        return Repeat(self.item.expand())

    def replace(self, old, new):
        return Repeat(self.item.replace(old, new))

    def remove_empty(self):
        item = self.item.remove_empty()
        if isinstance(item, Empty):
            return Empty()
        return Repeat(item)


@dataclass(frozen=True)
class Option(Item):
    item: Item

    def to_pest(self):
        if isinstance(self.item, (Choice, Sequence)):
            return f"({self.item.to_pest()})?"
        else:
            return f"{self.item.to_pest()}?"

    def first(self):
        return self.item.first() | {None}

    def expand(self):
        return Choice([self.item.expand(), Empty()]).expand()

    def replace(self, old, new):
        return Option(self.item.replace(old, new))

    def remove_empty(self):
        item = self.item.remove_empty()
        if isinstance(item, Empty):
            return Empty()
        return Option(item)


@dataclass(frozen=True)
class Name(Item):
    name: str

    def to_pest(self):
        return self.name


@dataclass(frozen=True)
class String(Item):
    value: str

    def to_pest(self):
        return self.value


@dataclass(frozen=True)
class Empty(Item):
    def to_pest(self):
        return '""'


def flatten(a):
    if not isinstance(a, Iterable):
        yield a
        return
    for i in a:
        if isinstance(a, Iterable):
            yield from flatten(i)
            continue
        yield i
