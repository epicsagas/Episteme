"""
Base classes for multi-language code analysis
"""

from abc import ABC, abstractmethod
from dataclasses import asdict, dataclass
from enum import Enum
from typing import Dict, List, Optional


class SmellType(Enum):
    """Code smell types — IDs must match relations.json catalog."""

    LONG_METHOD = "Long Method"  # SMELL-01
    LONG_PARAMETER_LIST = "Long Parameter List"  # SMELL-02
    PRIMITIVE_OBSESSION = "Primitive Obsession"  # SMELL-03
    LARGE_CLASS = "Large Class"  # SMELL-04
    DATA_CLUMPS = "Data Clumps"  # SMELL-05
    SWITCH_STATEMENTS = "Switch Statements"  # SMELL-06
    DATA_CLASS = "Data Class"  # SMELL-07
    SHOTGUN_SURGERY = "Shotgun Surgery"  # SMELL-09
    DIVERGENT_CHANGE = "Divergent Change"  # SMELL-10
    LAZY_CLASS = "Lazy Class"  # SMELL-11
    SPECULATIVE_GENERALITY = "Speculative Generality"  # SMELL-12
    DUPLICATE_CODE = "Duplicate Code"  # SMELL-13
    MIDDLE_MAN = "Middle Man"  # SMELL-14
    FEATURE_ENVY = "Feature Envy"  # SMELL-18
    MESSAGE_CHAINS = "Message Chains"  # SMELL-20
    GOD_OBJECT = "God Object"  # SMELL-21


@dataclass
class CodeMetrics:
    """Code metrics for a function/class"""

    loc: int  # Lines of code
    cyclomatic_complexity: int  # CC
    nesting_depth: int
    parameter_count: int
    local_variables: int
    return_statements: int
    method_count: int = 0  # For classes
    field_count: int = 0  # For classes
    external_calls: int = 0  # For Feature Envy
    primitive_params: int = 0  # For Primitive Obsession
    branch_count: int = 0  # For Switch Statements (if/elif/switch branches)
    method_call_chains: int = 0  # For Message Chains (a.b().c().d())
    delegation_methods: int = 0  # For Middle Man (methods that only delegate)
    ast_hash: str = ""  # For Duplicate Code detection

    def to_dict(self) -> Dict:
        return asdict(self)


@dataclass
class SmellDetection:
    """Detected code smell with confidence and location"""

    smell_id: str
    smell_name: str
    confidence: float  # 0.0 - 1.0
    location: str  # file:line
    function_name: str
    metrics: CodeMetrics
    reasons: List[str]

    def to_dict(self) -> Dict:
        return {
            "smell_id": self.smell_id,
            "smell_name": self.smell_name,
            "confidence": self.confidence,
            "location": self.location,
            "function_name": self.function_name,
            "metrics": self.metrics.to_dict(),
            "reasons": self.reasons,
        }


class LanguageParser(ABC):
    """Abstract base class for language-specific parsers"""

    @abstractmethod
    def parse_file(self, file_path: str) -> List[SmellDetection]:
        """Parse a file and detect code smells"""
        pass

    @abstractmethod
    def parse_code(self, code: str, file_name: str = "temp") -> List[SmellDetection]:
        """Parse code string and detect code smells"""
        pass

    @abstractmethod
    def get_supported_extensions(self) -> List[str]:
        """Get file extensions this parser supports"""
        pass

    def detect_long_method(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """Generic long method detection"""
        reasons = []
        confidence = 0.0

        if metrics.loc > 50:
            reasons.append(f"LOC={metrics.loc} exceeds 50")
            confidence += 0.3
        elif metrics.loc > 30:
            reasons.append(f"LOC={metrics.loc} exceeds 30")
            confidence += 0.15

        if metrics.cyclomatic_complexity > 15:
            reasons.append(f"CC={metrics.cyclomatic_complexity} exceeds 15")
            confidence += 0.4
        elif metrics.cyclomatic_complexity > 10:
            reasons.append(f"CC={metrics.cyclomatic_complexity} exceeds 10")
            confidence += 0.25

        if metrics.nesting_depth > 4:
            reasons.append(f"Nesting depth={metrics.nesting_depth} exceeds 4")
            confidence += 0.2
        elif metrics.nesting_depth > 3:
            reasons.append(f"Nesting depth={metrics.nesting_depth} exceeds 3")
            confidence += 0.1

        if confidence >= 0.5:
            return SmellDetection(
                smell_id="SMELL-01",
                smell_name=SmellType.LONG_METHOD.value,
                confidence=min(confidence, 1.0),
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=reasons,
            )
        return None

    def detect_long_parameter_list(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """Generic long parameter list detection"""
        if metrics.parameter_count <= 4:
            return None

        confidence = 0.0
        reasons = []

        if metrics.parameter_count > 7:
            reasons.append(f"Parameter count={metrics.parameter_count} exceeds 7")
            confidence = 0.95
        elif metrics.parameter_count > 5:
            reasons.append(f"Parameter count={metrics.parameter_count} exceeds 5")
            confidence = 0.80
        else:
            reasons.append(f"Parameter count={metrics.parameter_count} exceeds 4")
            confidence = 0.65

        return SmellDetection(
            smell_id="SMELL-02",
            smell_name=SmellType.LONG_PARAMETER_LIST.value,
            confidence=confidence,
            location=location,
            function_name=name,
            metrics=metrics,
            reasons=reasons,
        )

    def detect_large_class(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """Detect Large Class smell"""
        reasons = []
        confidence = 0.0

        if metrics.method_count > 20:
            reasons.append(f"Method count={metrics.method_count} exceeds 20")
            confidence += 0.4
        elif metrics.method_count > 15:
            reasons.append(f"Method count={metrics.method_count} exceeds 15")
            confidence += 0.2

        if metrics.field_count > 15:
            reasons.append(f"Field count={metrics.field_count} exceeds 15")
            confidence += 0.3
        elif metrics.field_count > 10:
            reasons.append(f"Field count={metrics.field_count} exceeds 10")
            confidence += 0.15

        if metrics.loc > 300:
            reasons.append(f"LOC={metrics.loc} exceeds 300")
            confidence += 0.3
        elif metrics.loc > 200:
            reasons.append(f"LOC={metrics.loc} exceeds 200")
            confidence += 0.15

        if confidence >= 0.5:
            return SmellDetection(
                smell_id="SMELL-04",
                smell_name=SmellType.LARGE_CLASS.value,
                confidence=min(confidence, 1.0),
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=reasons,
            )
        return None

    def detect_data_class(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """Detect Data Class smell"""
        if metrics.method_count == 0:
            return None

        # Data class has many fields but few/no behavior methods
        # Heuristic: if field_count >= method_count, likely data class
        getter_setter_ratio = metrics.field_count / metrics.method_count

        if getter_setter_ratio >= 2.0 and metrics.field_count >= 5:
            return SmellDetection(
                smell_id="SMELL-07",
                smell_name=SmellType.DATA_CLASS.value,
                confidence=0.75,
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=[
                    f"High field-to-method ratio ({getter_setter_ratio:.1f})",
                    f"Field count={metrics.field_count}, few behavior methods",
                ],
            )
        return None

    def detect_lazy_class(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """Detect Lazy Class smell"""
        if metrics.loc < 20 and metrics.method_count <= 2:
            return SmellDetection(
                smell_id="SMELL-11",
                smell_name=SmellType.LAZY_CLASS.value,
                confidence=0.70,
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=[
                    f"LOC={metrics.loc} is very small",
                    f"Method count={metrics.method_count}, minimal functionality",
                ],
            )
        return None

    def detect_duplicate_code(
        self,
        metrics: CodeMetrics,
        location: str,
        name: str,
        all_hashes: Optional[Dict[str, List[str]]] = None,
    ) -> Optional[SmellDetection]:
        """
        Detect Duplicate Code smell using AST hash comparison.

        TODO: Full implementation requires cross-file analysis.
        Current MVP version checks if ast_hash is provided and matches known duplicates.

        Args:
            metrics: Code metrics including ast_hash
            location: Source location
            name: Function/class name
            all_hashes: Optional dict of {ast_hash: [locations]} for duplicate detection
        """
        if not metrics.ast_hash or not all_hashes:
            return None

        # Check if this hash appears multiple times
        duplicate_locations = all_hashes.get(metrics.ast_hash, [])
        if len(duplicate_locations) > 1:
            other_locations = [loc for loc in duplicate_locations if loc != location]
            if other_locations:
                confidence = min(0.95, 0.7 + (len(duplicate_locations) - 1) * 0.1)
                return SmellDetection(
                    smell_id="SMELL-13",
                    smell_name=SmellType.DUPLICATE_CODE.value,
                    confidence=confidence,
                    location=location,
                    function_name=name,
                    metrics=metrics,
                    reasons=[
                        f"Code duplicated in {len(duplicate_locations)} locations",
                        f"Also found at: {', '.join(other_locations[:3])}",
                    ],
                )
        return None

    def detect_god_object(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """
        Detect God Object smell - class with too many responsibilities.

        A God Object has excessive methods, fields, and LOC, indicating it does too much.
        This is a more severe form of Large Class with multiple responsibility indicators.
        """
        reasons = []
        confidence = 0.0

        # God Object has VERY high counts across multiple dimensions
        if metrics.method_count > 30:
            reasons.append(f"Excessive method count={metrics.method_count} (>30)")
            confidence += 0.35
        elif metrics.method_count > 25:
            reasons.append(f"Very high method count={metrics.method_count} (>25)")
            confidence += 0.2

        if metrics.field_count > 20:
            reasons.append(f"Excessive field count={metrics.field_count} (>20)")
            confidence += 0.35
        elif metrics.field_count > 15:
            reasons.append(f"Very high field count={metrics.field_count} (>15)")
            confidence += 0.2

        if metrics.loc > 500:
            reasons.append(f"Excessive LOC={metrics.loc} (>500)")
            confidence += 0.3
        elif metrics.loc > 400:
            reasons.append(f"Very high LOC={metrics.loc} (>400)")
            confidence += 0.15

        # Additional indicator: high cyclomatic complexity suggests complex logic
        if metrics.cyclomatic_complexity > 50:
            reasons.append(f"Extreme complexity CC={metrics.cyclomatic_complexity}")
            confidence += 0.2

        if confidence >= 0.6:
            return SmellDetection(
                smell_id="SMELL-21",
                smell_name=SmellType.GOD_OBJECT.value,
                confidence=min(confidence, 1.0),
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=reasons,
            )
        return None

    def detect_switch_statements(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """
        Detect Switch Statements smell - long if-elif chains or switch/match blocks.

        Excessive branching (>5 branches) suggests the need for polymorphism.
        """
        if metrics.branch_count <= 5:
            return None

        confidence = 0.0
        reasons = []

        if metrics.branch_count > 10:
            reasons.append(f"Excessive branching with {metrics.branch_count} branches (>10)")
            confidence = 0.90
        elif metrics.branch_count > 7:
            reasons.append(f"High branching with {metrics.branch_count} branches (>7)")
            confidence = 0.75
        else:
            reasons.append(f"Many branches ({metrics.branch_count}) suggest need for polymorphism")
            confidence = 0.60

        # Additional factor: high CC combined with many branches is worse
        if metrics.cyclomatic_complexity > 15:
            reasons.append(f"Combined with high CC={metrics.cyclomatic_complexity}")
            confidence = min(confidence + 0.15, 1.0)

        return SmellDetection(
            smell_id="SMELL-06",
            smell_name=SmellType.SWITCH_STATEMENTS.value,
            confidence=confidence,
            location=location,
            function_name=name,
            metrics=metrics,
            reasons=reasons,
        )

    def detect_primitive_obsession(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """
        Detect Primitive Obsession smell - functions with many primitive parameters.

        Using primitives (int, str, bool) instead of small objects for domain concepts.
        """
        if metrics.primitive_params < 3:
            return None

        # Calculate ratio of primitive to total parameters
        primitive_ratio = metrics.primitive_params / max(metrics.parameter_count, 1)

        confidence = 0.0
        reasons = []

        if metrics.primitive_params >= 5 and primitive_ratio >= 0.8:
            reasons.append(f"{metrics.primitive_params} primitive parameters (>=5)")
            reasons.append(f"{primitive_ratio:.0%} of parameters are primitives")
            confidence = 0.85
        elif metrics.primitive_params >= 4 and primitive_ratio >= 0.75:
            reasons.append(f"{metrics.primitive_params} primitive parameters")
            reasons.append(f"High primitive ratio {primitive_ratio:.0%}")
            confidence = 0.70
        elif metrics.primitive_params >= 3 and primitive_ratio >= 0.7:
            reasons.append(
                f"{metrics.primitive_params} primitive parameters suggest domain object needed"
            )
            confidence = 0.55
        else:
            return None

        return SmellDetection(
            smell_id="SMELL-03",
            smell_name=SmellType.PRIMITIVE_OBSESSION.value,
            confidence=confidence,
            location=location,
            function_name=name,
            metrics=metrics,
            reasons=reasons,
        )

    def detect_shotgun_surgery(
        self, metrics: CodeMetrics, location: str, name: str, dependency_count: int = 0
    ) -> Optional[SmellDetection]:
        """
        Detect Shotgun Surgery smell - changes require modifications across many files.

        TODO: Full implementation requires cross-file dependency analysis.
        Current MVP version uses dependency_count as a heuristic when provided.

        Args:
            metrics: Code metrics
            location: Source location
            name: Function/class name
            dependency_count: Number of files that depend on this code (from external analysis)
        """
        if dependency_count == 0:
            # No cross-file analysis available, skip detection
            return None

        # High fan-out suggests changes here will require many file updates
        if dependency_count >= 10:
            return SmellDetection(
                smell_id="SMELL-09",
                smell_name=SmellType.SHOTGUN_SURGERY.value,
                confidence=0.80,
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=[
                    f"Used by {dependency_count} different files",
                    "Changes here will require widespread modifications",
                ],
            )
        elif dependency_count >= 7:
            return SmellDetection(
                smell_id="SMELL-09",
                smell_name=SmellType.SHOTGUN_SURGERY.value,
                confidence=0.65,
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=[
                    f"Used by {dependency_count} files",
                    "Moderate coupling suggests refactoring risk",
                ],
            )

        return None

    def detect_divergent_change(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """
        Detect Divergent Change smell - class changes for multiple unrelated reasons.

        Heuristic: High CC combined with many methods suggests multiple responsibilities.
        Each responsibility might change for different reasons.
        """
        # Needs both complexity and size to indicate multiple change reasons
        if metrics.cyclomatic_complexity <= 15 or metrics.method_count <= 8:
            return None

        reasons = []
        confidence = 0.0

        # High complexity + many methods = likely multiple responsibilities
        if metrics.cyclomatic_complexity > 25 and metrics.method_count > 15:
            reasons.append(
                f"High CC={metrics.cyclomatic_complexity} with {metrics.method_count} methods"
            )
            reasons.append("Multiple responsibilities suggest multiple change reasons")
            confidence = 0.80
        elif metrics.cyclomatic_complexity > 20 and metrics.method_count > 12:
            reasons.append(
                f"CC={metrics.cyclomatic_complexity} with {metrics.method_count} methods"
            )
            reasons.append("Likely has multiple change reasons")
            confidence = 0.65
        elif metrics.cyclomatic_complexity > 15 and metrics.method_count > 8:
            reasons.append(f"Moderate CC={metrics.cyclomatic_complexity} and method count")
            confidence = 0.55

        if confidence >= 0.55:
            # Additional evidence: high field count suggests data-driven multiple concerns
            if metrics.field_count > 10:
                reasons.append(f"Many fields ({metrics.field_count}) reinforce multiple concerns")
                confidence = min(confidence + 0.1, 1.0)

            return SmellDetection(
                smell_id="SMELL-10",
                smell_name=SmellType.DIVERGENT_CHANGE.value,
                confidence=confidence,
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=reasons,
            )
        return None

    def detect_speculative_generality(
        self,
        metrics: CodeMetrics,
        location: str,
        name: str,
        subclass_count: int = 0,
        usage_count: int = 0,
    ) -> Optional[SmellDetection]:
        """
        Detect Speculative Generality smell - unused abstraction.

        Classes with only one subclass or interfaces with single implementation
        suggest over-engineering for future flexibility that hasn't materialized.

        TODO: Full implementation requires inheritance analysis across codebase.
        Current MVP version uses subclass_count and usage_count when provided.

        Args:
            metrics: Code metrics
            location: Source location
            name: Function/class name
            subclass_count: Number of subclasses (0 if not abstract/base class)
            usage_count: Number of actual usages in codebase
        """
        reasons = []
        confidence = 0.0

        # Abstract class or interface with only one implementation
        if subclass_count == 1:
            reasons.append("Abstract class/interface with only one implementation")
            reasons.append("Abstraction may be premature/unnecessary")
            confidence = 0.75

        # Class exists but is rarely or never used
        if usage_count == 0 and metrics.method_count > 0:
            reasons.append("Class is defined but never used")
            confidence = max(confidence, 0.85)
        elif usage_count == 1 and metrics.method_count > 3:
            reasons.append("Complex class with only one usage point")
            confidence = max(confidence, 0.60)

        # Combination: abstract with single impl AND low usage
        if subclass_count == 1 and usage_count <= 1:
            confidence = 0.90

        if confidence >= 0.6:
            return SmellDetection(
                smell_id="SMELL-12",
                smell_name=SmellType.SPECULATIVE_GENERALITY.value,
                confidence=confidence,
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=reasons if reasons else ["Unused or over-engineered abstraction"],
            )
        return None

    def detect_message_chains(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """
        Detect Message Chains smell - long method call chains like a.b().c().d().

        Violates Law of Demeter and creates tight coupling.
        """
        if metrics.method_call_chains <= 2:
            return None

        confidence = 0.0
        reasons = []

        if metrics.method_call_chains > 5:
            reasons.append(f"Very long call chains (depth={metrics.method_call_chains})")
            reasons.append("Violates Law of Demeter, creates tight coupling")
            confidence = 0.90
        elif metrics.method_call_chains > 4:
            reasons.append(f"Long call chains (depth={metrics.method_call_chains})")
            reasons.append("Consider introducing intermediate methods")
            confidence = 0.75
        elif metrics.method_call_chains > 2:
            reasons.append(f"Call chain depth={metrics.method_call_chains} suggests coupling")
            confidence = 0.60

        return SmellDetection(
            smell_id="SMELL-20",
            smell_name=SmellType.MESSAGE_CHAINS.value,
            confidence=confidence,
            location=location,
            function_name=name,
            metrics=metrics,
            reasons=reasons,
        )

    def detect_middle_man(
        self, metrics: CodeMetrics, location: str, name: str
    ) -> Optional[SmellDetection]:
        """
        Detect Middle Man smell - class where most methods just delegate.

        If >70% of methods are single-line delegations, the class adds no value.
        """
        if metrics.method_count == 0 or metrics.delegation_methods == 0:
            return None

        delegation_ratio = metrics.delegation_methods / metrics.method_count

        if delegation_ratio > 0.7 and metrics.method_count >= 3:
            confidence = 0.85 if delegation_ratio > 0.85 else 0.70
            return SmellDetection(
                smell_id="SMELL-14",
                smell_name=SmellType.MIDDLE_MAN.value,
                confidence=confidence,
                location=location,
                function_name=name,
                metrics=metrics,
                reasons=[
                    f"{metrics.delegation_methods}/{metrics.method_count} methods are simple delegations",
                    f"Delegation ratio: {delegation_ratio:.0%}",
                    "Class adds little value, consider removing",
                ],
            )
        return None
