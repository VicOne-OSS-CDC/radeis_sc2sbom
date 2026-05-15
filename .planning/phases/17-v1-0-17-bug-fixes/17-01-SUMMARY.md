<phase>17</phase>
<plan>01</plan>
<title>AUTOSAR arxml parser and version extraction bug fixes</title>

<one_liner>Fixed BUG-01/02/03: arxml parser now extracts SW-COMPONENT-PROTOTYPE, BSW-MODULE-DESCRIPTION, and SWC type definitions; .epd REVISION-LABEL and Doxygen SW Version populate real versions; system linker deps shadowed by autosar ecosystem entries are deduped and upgraded</one_liner>

<status>complete</status>

<what_was_done>
- BUG-01 (8d3f961): Created src/parsers/c/arxml.rs — extracts SHORT-NAME from SW-COMPONENT-PROTOTYPE and BSW-MODULE-DESCRIPTION elements as autosar-ecosystem dependencies. Wired into scanner loop behind is_autosar guard. AUTOSAR-SOFTWARE-DEMO went from 0 to 2 deps.
- BUG-02 (4022e3b): Extended arxml parser to extract APPLICATION-SW-COMPONENT-TYPE, ECU-ABSTRACTION-SW-COMPONENT-TYPE, SERVICE-SW-COMPONENT-TYPE, COMPOSITION-SW-COMPONENT-TYPE, COMPLEX-DEVICE-DRIVER-SW-COMPONENT-TYPE. AUTOSAR-SOFTWARE-DEMO now reports 5 deps.
- BUG-03 (14e60a5): .epd parser extracts ECUC-MODULE-DEF REVISION-LABEL; Doxygen C/H header parser extracts SW Version. Both populate version fields. AUTOSAR_SampleProject_S32K144 now shows 17 of 18 components with real versions.
- Dedup fix (b15067b): System deps with same name as an autosar-ecosystem dep are now suppressed. Duplicate unspecified entries dropped when versioned entry exists.
- Ecosystem upgrade (9ba0c2b, d8ca713): Post-walk pass matches system linker deps (-lAdc, -lGpt, SWC dirs) against epd_versions and doxygen versions, promoting them to autosar ecosystem with real version strings. AUTOSAR_SampleProject_S32K144 scan regenerated showing 18 components.
</what_was_done>

<commits>
- 8d3f961 fix(17/BUG-01): parse .arxml files for AUTOSAR SW component dependencies
- 4022e3b fix(17/BUG-02): extract SWC type definition elements from arxml files
- 14e60a5 fix(autosar): extract real versions from .epd and Doxygen headers (BUG-03)
- b15067b fix(dedup): drop system linker deps shadowed by autosar ecosystem entries
- 9ba0c2b fix(autosar): upgrade system linker deps to autosar ecosystem using epd versions
- d8ca713 fix(autosar): also upgrade system deps with doxygen versions to autosar ecosystem
</commits>
