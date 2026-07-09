# Mycelium Lib Index — the self-hosted `.myc` reference

> **Honesty:** `Empirical/Declared — line/regex heuristic over .myc source (mirrors tools/docgen/code_index.py's approach one level up the stack); source is ground truth. Use this index to find where to Read, not as an authoritative reference.`
> Use the index to find where to `Read`, not as an authoritative reference.

## compiler

### compiler.ambient

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.ambient` | nodule | `lib/compiler/ambient.myc:16` | `nodule compiler.ambient` | Self-hosted ambient-representation resolution (M-740 Stage 4; DN-26 §7.3 row 4). A faithful port of crates/mycelium-l1/src/ambient.rs (RFC-0012 §4.3/§4.4): a surface->surface "expand to longhand" pass (`resolve`/`resolve_report`) plus the canonical pretty-printer (`expand_to_source`/`expand_phylum_to_source`), over a SELF-CONTAINED copy of the AST vocabulary (mirroring compiler.ast) — per M-982, cross-nodule EXECUTION is still staged, so this nodule cannot `use compiler.ast.\*` and RUN it; it redeclares the full AST it dispatches over (Item/Expr/BaseType/Literal/Pattern, TWICE — once for the resolver, once for the printer), matching crate::ambient's own `resolve(&Nodule) -> Result<Nodule, AmbientError>` / `expand_to_source(&Nodule) -> String` shape directly. | Empirical/Declared |
| `compiler.ambient::Option` | type | `lib/compiler/ambient.myc:86` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.ambient::Option::None` | ctor | `lib/compiler/ambient.myc:86` | `None` | — | Empirical/Declared |
| `compiler.ambient::Option::Some` | ctor | `lib/compiler/ambient.myc:86` | `Some(A)` | — | Empirical/Declared |
| `compiler.ambient::Result` | type | `lib/compiler/ambient.myc:87` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `compiler.ambient::Result::Err` | ctor | `lib/compiler/ambient.myc:87` | `Err(E)` | — | Empirical/Declared |
| `compiler.ambient::Result::Ok` | ctor | `lib/compiler/ambient.myc:87` | `Ok(A)` | — | Empirical/Declared |
| `compiler.ambient::Vec` | type | `lib/compiler/ambient.myc:88` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.ambient::Vec::Cons` | ctor | `lib/compiler/ambient.myc:88` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.ambient::Vec::Nil` | ctor | `lib/compiler/ambient.myc:88` | `Nil` | — | Empirical/Declared |
| `compiler.ambient::Pair` | type | `lib/compiler/ambient.myc:89` | `type Pair[A, B] = Pr(A, B)` | — | Empirical/Declared |
| `compiler.ambient::Pair::Pr` | ctor | `lib/compiler/ambient.myc:89` | `Pr(A, B)` | — | Empirical/Declared |
| `compiler.ambient::rev_acc` | fn | `lib/compiler/ambient.myc:93` | `fn rev_acc[A](xs: Vec[A], acc: Vec[A]) => Vec[A]` | rev_acc: the direct-tail list-reversal underpinning every accumulator+reverse list-building loop below (M-985) — `rev_acc(xs, acc)` = reverse(xs) ++ acc, one direct tail call per cell. | Empirical/Declared |
| `compiler.ambient::Vis` | type | `lib/compiler/ambient.myc:97` | `type Vis = Private \| Pub` | — | Empirical/Declared |
| `compiler.ambient::Vis::Private` | ctor | `lib/compiler/ambient.myc:97` | `Private` | — | Empirical/Declared |
| `compiler.ambient::Vis::Pub` | ctor | `lib/compiler/ambient.myc:97` | `Pub` | — | Empirical/Declared |
| `compiler.ambient::Path` | type | `lib/compiler/ambient.myc:98` | `type Path = Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.ambient::Path::Pth` | ctor | `lib/compiler/ambient.myc:98` | `Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.ambient::UsePath` | type | `lib/compiler/ambient.myc:99` | `type UsePath = UP(Path, Bool)` | — | Empirical/Declared |
| `compiler.ambient::UsePath::UP` | ctor | `lib/compiler/ambient.myc:99` | `UP(Path, Bool)` | — | Empirical/Declared |
| `compiler.ambient::Paradigm` | type | `lib/compiler/ambient.myc:100` | `type Paradigm = PBinary \| PTernary \| PDense \| PVsa` | — | Empirical/Declared |
| `compiler.ambient::Paradigm::PBinary` | ctor | `lib/compiler/ambient.myc:100` | `PBinary` | — | Empirical/Declared |
| `compiler.ambient::Paradigm::PDense` | ctor | `lib/compiler/ambient.myc:100` | `PDense` | — | Empirical/Declared |
| `compiler.ambient::Paradigm::PTernary` | ctor | `lib/compiler/ambient.myc:100` | `PTernary` | — | Empirical/Declared |
| `compiler.ambient::Paradigm::PVsa` | ctor | `lib/compiler/ambient.myc:100` | `PVsa` | — | Empirical/Declared |
| `compiler.ambient::Scalar` | type | `lib/compiler/ambient.myc:101` | `type Scalar = SF16 \| SBf16 \| SF32 \| SF64` | — | Empirical/Declared |
| `compiler.ambient::Scalar::SBf16` | ctor | `lib/compiler/ambient.myc:101` | `SBf16` | — | Empirical/Declared |
| `compiler.ambient::Scalar::SF16` | ctor | `lib/compiler/ambient.myc:101` | `SF16` | — | Empirical/Declared |
| `compiler.ambient::Scalar::SF32` | ctor | `lib/compiler/ambient.myc:101` | `SF32` | — | Empirical/Declared |
| `compiler.ambient::Scalar::SF64` | ctor | `lib/compiler/ambient.myc:101` | `SF64` | — | Empirical/Declared |
| `compiler.ambient::Sparsity` | type | `lib/compiler/ambient.myc:102` | `type Sparsity = SpDense \| SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::Sparsity::SpDense` | ctor | `lib/compiler/ambient.myc:102` | `SpDense` | — | Empirical/Declared |
| `compiler.ambient::Sparsity::SpSparse` | ctor | `lib/compiler/ambient.myc:102` | `SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::AmbientParams` | type | `lib/compiler/ambient.myc:103` | `type AmbientParams = APSize(Binary{32}) \| APDense(Binary{32}, Scalar) \| APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.ambient::AmbientParams::APSize` | ctor | `lib/compiler/ambient.myc:104` | `APSize(Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::AmbientParams::APDense` | ctor | `lib/compiler/ambient.myc:105` | `APDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.ambient::AmbientParams::APVsa` | ctor | `lib/compiler/ambient.myc:106` | `APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.ambient::Strength` | type | `lib/compiler/ambient.myc:107` | `type Strength = GExact \| GProven \| GEmpirical \| GDeclared` | — | Empirical/Declared |
| `compiler.ambient::Strength::GDeclared` | ctor | `lib/compiler/ambient.myc:107` | `GDeclared` | — | Empirical/Declared |
| `compiler.ambient::Strength::GEmpirical` | ctor | `lib/compiler/ambient.myc:107` | `GEmpirical` | — | Empirical/Declared |
| `compiler.ambient::Strength::GExact` | ctor | `lib/compiler/ambient.myc:107` | `GExact` | — | Empirical/Declared |
| `compiler.ambient::Strength::GProven` | ctor | `lib/compiler/ambient.myc:107` | `GProven` | — | Empirical/Declared |
| `compiler.ambient::WidthRef` | type | `lib/compiler/ambient.myc:108` | `type WidthRef = WLit(Binary{32}) \| WName(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::WidthRef::WLit` | ctor | `lib/compiler/ambient.myc:108` | `WLit(Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::WidthRef::WName` | ctor | `lib/compiler/ambient.myc:108` | `WName(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::ParamKind` | type | `lib/compiler/ambient.myc:109` | `type ParamKind = PkType \| PkWidth` | — | Empirical/Declared |
| `compiler.ambient::ParamKind::PkType` | ctor | `lib/compiler/ambient.myc:109` | `PkType` | — | Empirical/Declared |
| `compiler.ambient::ParamKind::PkWidth` | ctor | `lib/compiler/ambient.myc:109` | `PkWidth` | — | Empirical/Declared |
| `compiler.ambient::TraitRef` | type | `lib/compiler/ambient.myc:110` | `type TraitRef = TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::TraitRef::TRf` | ctor | `lib/compiler/ambient.myc:110` | `TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::TypeParam` | type | `lib/compiler/ambient.myc:111` | `type TypeParam = TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.ambient::TypeParam::TP` | ctor | `lib/compiler/ambient.myc:111` | `TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.ambient::EffectBudget` | type | `lib/compiler/ambient.myc:112` | `type EffectBudget = EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.ambient::EffectBudget::EB` | ctor | `lib/compiler/ambient.myc:112` | `EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.ambient::FnSig` | type | `lib/compiler/ambient.myc:113` | `type FnSig = FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.ambient::FnSig::FS` | ctor | `lib/compiler/ambient.myc:113` | `FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.ambient::Param` | type | `lib/compiler/ambient.myc:114` | `type Param = Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.ambient::Param::Prm` | ctor | `lib/compiler/ambient.myc:114` | `Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.ambient::TypeRef` | type | `lib/compiler/ambient.myc:115` | `type TypeRef = TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.ambient::TypeRef::TR` | ctor | `lib/compiler/ambient.myc:115` | `TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.ambient::BaseType` | type | `lib/compiler/ambient.myc:116` | `type BaseType = KwBinary(WidthRef) \| KwTernary(WidthRef) \| KwDense(Binary{32}, Scalar) \| Vsa(Bytes, Binary{32}, Sparsity) \| KwSubstrate(Bytes) \| KwSeq(TypeRef, Binary{32}) \| KwBytes \| KwFloat \| Named(Bytes, Vec[TypeRef]) \| Ambient(AmbientParams) \| FnArrow(TypeRef, TypeRef) \| Tuple(Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::BaseType::KwBinary` | ctor | `lib/compiler/ambient.myc:117` | `KwBinary(WidthRef)` | — | Empirical/Declared |
| `compiler.ambient::BaseType::KwTernary` | ctor | `lib/compiler/ambient.myc:118` | `KwTernary(WidthRef)` | — | Empirical/Declared |
| `compiler.ambient::BaseType::KwDense` | ctor | `lib/compiler/ambient.myc:119` | `KwDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.ambient::BaseType::Vsa` | ctor | `lib/compiler/ambient.myc:120` | `Vsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.ambient::BaseType::KwSubstrate` | ctor | `lib/compiler/ambient.myc:121` | `KwSubstrate(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::BaseType::KwSeq` | ctor | `lib/compiler/ambient.myc:122` | `KwSeq(TypeRef, Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::BaseType::KwBytes` | ctor | `lib/compiler/ambient.myc:123` | `KwBytes` | — | Empirical/Declared |
| `compiler.ambient::BaseType::KwFloat` | ctor | `lib/compiler/ambient.myc:124` | `KwFloat` | — | Empirical/Declared |
| `compiler.ambient::BaseType::Named` | ctor | `lib/compiler/ambient.myc:125` | `Named(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::BaseType::Ambient` | ctor | `lib/compiler/ambient.myc:126` | `Ambient(AmbientParams)` | — | Empirical/Declared |
| `compiler.ambient::BaseType::FnArrow` | ctor | `lib/compiler/ambient.myc:127` | `FnArrow(TypeRef, TypeRef)` | — | Empirical/Declared |
| `compiler.ambient::BaseType::Tuple` | ctor | `lib/compiler/ambient.myc:128` | `Tuple(Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::ExecutionMode` | type | `lib/compiler/ambient.myc:129` | `type ExecutionMode = Interpreted \| Compiled` | — | Empirical/Declared |
| `compiler.ambient::ExecutionMode::Compiled` | ctor | `lib/compiler/ambient.myc:129` | `Compiled` | — | Empirical/Declared |
| `compiler.ambient::ExecutionMode::Interpreted` | ctor | `lib/compiler/ambient.myc:129` | `Interpreted` | — | Empirical/Declared |
| `compiler.ambient::FnDecl` | type | `lib/compiler/ambient.myc:130` | `type FnDecl = FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.ambient::FnDecl::FD` | ctor | `lib/compiler/ambient.myc:130` | `FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Ctor` | type | `lib/compiler/ambient.myc:131` | `type Ctor = Ctr(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::Ctor::Ctr` | ctor | `lib/compiler/ambient.myc:131` | `Ctr(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::TypeDecl` | type | `lib/compiler/ambient.myc:132` | `type TypeDecl = TD(Vis, Bytes, Vec[Bytes], Vec[Ctor])` | — | Empirical/Declared |
| `compiler.ambient::TypeDecl::TD` | ctor | `lib/compiler/ambient.myc:132` | `TD(Vis, Bytes, Vec[Bytes], Vec[Ctor])` | — | Empirical/Declared |
| `compiler.ambient::TraitDecl` | type | `lib/compiler/ambient.myc:133` | `type TraitDecl = TrD(Vis, Bytes, Vec[Bytes], Vec[FnSig])` | — | Empirical/Declared |
| `compiler.ambient::TraitDecl::TrD` | ctor | `lib/compiler/ambient.myc:133` | `TrD(Vis, Bytes, Vec[Bytes], Vec[FnSig])` | — | Empirical/Declared |
| `compiler.ambient::ImplDecl` | type | `lib/compiler/ambient.myc:134` | `type ImplDecl = ImD(Bytes, Vec[TypeRef], TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ambient::ImplDecl::ImD` | ctor | `lib/compiler/ambient.myc:134` | `ImD(Bytes, Vec[TypeRef], TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ambient::ViaDecl` | type | `lib/compiler/ambient.myc:135` | `type ViaDecl = VD(Binary{32}, Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::ViaDecl::VD` | ctor | `lib/compiler/ambient.myc:135` | `VD(Binary{32}, Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ambient::ObjectDecl` | type | `lib/compiler/ambient.myc:136` | `type ObjectDecl = OD(Vis, Bytes, Vec[Bytes], Ctor, Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ambient::ObjectDecl::OD` | ctor | `lib/compiler/ambient.myc:136` | `OD(Vis, Bytes, Vec[Bytes], Ctor, Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ambient::InherentImplDecl` | type | `lib/compiler/ambient.myc:137` | `type InherentImplDecl = IID(TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ambient::InherentImplDecl::IID` | ctor | `lib/compiler/ambient.myc:137` | `IID(TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ambient::LowerRhs` | type | `lib/compiler/ambient.myc:138` | `type LowerRhs = LRExpr(Expr) \| LRImpl(ImplDecl)` | — | Empirical/Declared |
| `compiler.ambient::LowerRhs::LRExpr` | ctor | `lib/compiler/ambient.myc:138` | `LRExpr(Expr)` | — | Empirical/Declared |
| `compiler.ambient::LowerRhs::LRImpl` | ctor | `lib/compiler/ambient.myc:138` | `LRImpl(ImplDecl)` | — | Empirical/Declared |
| `compiler.ambient::LowerDecl` | type | `lib/compiler/ambient.myc:139` | `type LowerDecl = LD(Bytes, Vec[Bytes], LowerRhs)` | — | Empirical/Declared |
| `compiler.ambient::LowerDecl::LD` | ctor | `lib/compiler/ambient.myc:139` | `LD(Bytes, Vec[Bytes], LowerRhs)` | — | Empirical/Declared |
| `compiler.ambient::DeriveDecl` | type | `lib/compiler/ambient.myc:140` | `type DeriveDecl = DD(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.ambient::DeriveDecl::DD` | ctor | `lib/compiler/ambient.myc:140` | `DD(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.ambient::Item` | type | `lib/compiler/ambient.myc:141` | `type Item = Use(UsePath) \| Default(Paradigm) \| Type(TypeDecl) \| Trait(TraitDecl) \| Impl(ImplDecl) \| Fn(FnDecl) \| Object(ObjectDecl) \| Lower(LowerDecl) \| Derive(DeriveDecl) \| InherentImpl(InherentImplDecl)` | — | Empirical/Declared |
| `compiler.ambient::Item::Use` | ctor | `lib/compiler/ambient.myc:142` | `Use(UsePath)` | — | Empirical/Declared |
| `compiler.ambient::Item::Default` | ctor | `lib/compiler/ambient.myc:143` | `Default(Paradigm)` | — | Empirical/Declared |
| `compiler.ambient::Item::Type` | ctor | `lib/compiler/ambient.myc:144` | `Type(TypeDecl)` | — | Empirical/Declared |
| `compiler.ambient::Item::Trait` | ctor | `lib/compiler/ambient.myc:145` | `Trait(TraitDecl)` | — | Empirical/Declared |
| `compiler.ambient::Item::Impl` | ctor | `lib/compiler/ambient.myc:146` | `Impl(ImplDecl)` | — | Empirical/Declared |
| `compiler.ambient::Item::Fn` | ctor | `lib/compiler/ambient.myc:147` | `Fn(FnDecl)` | — | Empirical/Declared |
| `compiler.ambient::Item::Object` | ctor | `lib/compiler/ambient.myc:148` | `Object(ObjectDecl)` | — | Empirical/Declared |
| `compiler.ambient::Item::Lower` | ctor | `lib/compiler/ambient.myc:149` | `Lower(LowerDecl)` | — | Empirical/Declared |
| `compiler.ambient::Item::Derive` | ctor | `lib/compiler/ambient.myc:150` | `Derive(DeriveDecl)` | — | Empirical/Declared |
| `compiler.ambient::Item::InherentImpl` | ctor | `lib/compiler/ambient.myc:151` | `InherentImpl(InherentImplDecl)` | — | Empirical/Declared |
| `compiler.ambient::Nodule` | type | `lib/compiler/ambient.myc:152` | `type Nodule = Nd(Path, Bool, Vec[Item])` | — | Empirical/Declared |
| `compiler.ambient::Nodule::Nd` | ctor | `lib/compiler/ambient.myc:152` | `Nd(Path, Bool, Vec[Item])` | — | Empirical/Declared |
| `compiler.ambient::Phylum` | type | `lib/compiler/ambient.myc:153` | `type Phylum = Phy(Option[Path], Vec[Nodule])` | — | Empirical/Declared |
| `compiler.ambient::Phylum::Phy` | ctor | `lib/compiler/ambient.myc:153` | `Phy(Option[Path], Vec[Nodule])` | — | Empirical/Declared |
| `compiler.ambient::Literal` | type | `lib/compiler/ambient.myc:154` | `type Literal = Bin(Bytes) \| Trit(Bytes) \| Int(Binary{64}) \| AmbientInt(Paradigm, Binary{64}) \| List(Vec[Expr]) \| LBytes(Bytes) \| Str(Bytes) \| LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::Literal::Bin` | ctor | `lib/compiler/ambient.myc:155` | `Bin(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::Literal::Trit` | ctor | `lib/compiler/ambient.myc:156` | `Trit(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::Literal::Int` | ctor | `lib/compiler/ambient.myc:157` | `Int(Binary{64})` | — | Empirical/Declared |
| `compiler.ambient::Literal::AmbientInt` | ctor | `lib/compiler/ambient.myc:158` | `AmbientInt(Paradigm, Binary{64})` | — | Empirical/Declared |
| `compiler.ambient::Literal::List` | ctor | `lib/compiler/ambient.myc:159` | `List(Vec[Expr])` | — | Empirical/Declared |
| `compiler.ambient::Literal::LBytes` | ctor | `lib/compiler/ambient.myc:160` | `LBytes(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::Literal::Str` | ctor | `lib/compiler/ambient.myc:161` | `Str(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::Literal::LFloat` | ctor | `lib/compiler/ambient.myc:162` | `LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::Pattern` | type | `lib/compiler/ambient.myc:163` | `type Pattern = PWildcard \| PLit(Literal) \| PCtor(Bytes, Vec[Pattern]) \| PIdent(Bytes) \| PTuple(Vec[Pattern]) \| POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.ambient::Pattern::PWildcard` | ctor | `lib/compiler/ambient.myc:164` | `PWildcard` | — | Empirical/Declared |
| `compiler.ambient::Pattern::PLit` | ctor | `lib/compiler/ambient.myc:165` | `PLit(Literal)` | — | Empirical/Declared |
| `compiler.ambient::Pattern::PCtor` | ctor | `lib/compiler/ambient.myc:166` | `PCtor(Bytes, Vec[Pattern])` | — | Empirical/Declared |
| `compiler.ambient::Pattern::PIdent` | ctor | `lib/compiler/ambient.myc:167` | `PIdent(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::Pattern::PTuple` | ctor | `lib/compiler/ambient.myc:168` | `PTuple(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.ambient::Pattern::POr` | ctor | `lib/compiler/ambient.myc:169` | `POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.ambient::Arm` | type | `lib/compiler/ambient.myc:170` | `type Arm = Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Arm::Ar` | ctor | `lib/compiler/ambient.myc:170` | `Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Hypha` | type | `lib/compiler/ambient.myc:171` | `type Hypha = Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.ambient::Hypha::Hy` | ctor | `lib/compiler/ambient.myc:171` | `Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr` | type | `lib/compiler/ambient.myc:172` | `type Expr = Let(Bytes, Option[TypeRef], Expr, Expr) \| If(Expr, Expr, Expr) \| Match(Expr, Vec[Arm]) \| For(Bytes, Expr, Bytes, Expr, Expr) \| Swap(Expr, TypeRef, Path) \| WithParadigm(Paradigm, Expr) \| Wild(Expr) \| Spore(Expr) \| Consume(Expr) \| Colony(Vec[Hypha]) \| Lambda(Vec[Param], Expr) \| App(Expr, Vec[Expr]) \| Fuse(Expr, Expr) \| Reclaim(Expr, Expr) \| Path(Path) \| Lit(Literal) \| Ascribe(Expr, TypeRef) \| TupleLit(Vec[Expr])` | — | Empirical/Declared |
| `compiler.ambient::Expr::Let` | ctor | `lib/compiler/ambient.myc:173` | `Let(Bytes, Option[TypeRef], Expr, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::If` | ctor | `lib/compiler/ambient.myc:174` | `If(Expr, Expr, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Match` | ctor | `lib/compiler/ambient.myc:175` | `Match(Expr, Vec[Arm])` | — | Empirical/Declared |
| `compiler.ambient::Expr::For` | ctor | `lib/compiler/ambient.myc:176` | `For(Bytes, Expr, Bytes, Expr, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Path` | ctor | `lib/compiler/ambient.myc:177` | `Path(Path)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Swap` | ctor | `lib/compiler/ambient.myc:177` | `Swap(Expr, TypeRef, Path)` | — | Empirical/Declared |
| `compiler.ambient::Expr::WithParadigm` | ctor | `lib/compiler/ambient.myc:178` | `WithParadigm(Paradigm, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Wild` | ctor | `lib/compiler/ambient.myc:179` | `Wild(Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Spore` | ctor | `lib/compiler/ambient.myc:180` | `Spore(Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Consume` | ctor | `lib/compiler/ambient.myc:181` | `Consume(Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Colony` | ctor | `lib/compiler/ambient.myc:182` | `Colony(Vec[Hypha])` | — | Empirical/Declared |
| `compiler.ambient::Expr::Lambda` | ctor | `lib/compiler/ambient.myc:183` | `Lambda(Vec[Param], Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::App` | ctor | `lib/compiler/ambient.myc:184` | `App(Expr, Vec[Expr])` | — | Empirical/Declared |
| `compiler.ambient::Expr::Fuse` | ctor | `lib/compiler/ambient.myc:185` | `Fuse(Expr, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Reclaim` | ctor | `lib/compiler/ambient.myc:186` | `Reclaim(Expr, Expr)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Lit` | ctor | `lib/compiler/ambient.myc:188` | `Lit(Literal)` | — | Empirical/Declared |
| `compiler.ambient::Expr::Ascribe` | ctor | `lib/compiler/ambient.myc:189` | `Ascribe(Expr, TypeRef)` | — | Empirical/Declared |
| `compiler.ambient::Expr::TupleLit` | ctor | `lib/compiler/ambient.myc:190` | `TupleLit(Vec[Expr])` | — | Empirical/Declared |
| `compiler.ambient::zero32` | fn | `lib/compiler/ambient.myc:199` | `fn zero32() => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::one32` | fn | `lib/compiler/ambient.myc:200` | `fn one32() => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::three32` | fn | `lib/compiler/ambient.myc:201` | `fn three32() => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::ten32` | fn | `lib/compiler/ambient.myc:202` | `fn ten32() => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::zero64` | fn | `lib/compiler/ambient.myc:203` | `fn zero64() => Binary{64}` | — | Empirical/Declared |
| `compiler.ambient::ten64` | fn | `lib/compiler/ambient.myc:204` | `fn ten64() => Binary{64}` | — | Empirical/Declared |
| `compiler.ambient::empty_bytes` | fn | `lib/compiler/ambient.myc:206` | `fn empty_bytes() => Bytes` | — | Empirical/Declared |
| `compiler.ambient::digit_table` | fn | `lib/compiler/ambient.myc:208` | `fn digit_table() => Bytes` | — | Empirical/Declared |
| `compiler.ambient::digit_bytes` | fn | `lib/compiler/ambient.myc:210` | `fn digit_bytes(d: Binary{32}) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::join_bytes` | fn | `lib/compiler/ambient.myc:213` | `fn join_bytes(xs: Vec[Bytes]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::u32_to_dec_acc` | fn | `lib/compiler/ambient.myc:216` | `fn u32_to_dec_acc(n: Binary{32}, acc: Vec[Bytes]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ambient::u32_to_dec` | fn | `lib/compiler/ambient.myc:226` | `fn u32_to_dec(n: Binary{32}) => Bytes` | u32_to_dec: decimal-render a Binary{32} (widths/dims/sparsity-k/the depth limit). Grounded (`Empirical`): the digit order falls out of prepending each new (more-significant) digit to the front of the accumulator as the value shrinks — no explicit reversal step is needed (verified during authoring against `u32_to_dec(0b...1111011)` (123) => "123"). | Empirical/Declared |
| `compiler.ambient::u64_to_dec_acc` | fn | `lib/compiler/ambient.myc:229` | `fn u64_to_dec_acc(n: Binary{64}, acc: Vec[Bytes]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ambient::u64_to_dec` | fn | `lib/compiler/ambient.myc:236` | `fn u64_to_dec(n: Binary{64}) => Bytes` | u64_to_dec: decimal-render a Binary{64} (`Literal::Int`/`AmbientInt` payloads). | Empirical/Declared |
| `compiler.ambient::backslash_b` | fn | `lib/compiler/ambient.myc:240` | `fn backslash_b() => Binary{8}` | — | Empirical/Declared |
| `compiler.ambient::quote_b` | fn | `lib/compiler/ambient.myc:241` | `fn quote_b() => Binary{8}` | — | Empirical/Declared |
| `compiler.ambient::nl_b` | fn | `lib/compiler/ambient.myc:242` | `fn nl_b() => Binary{8}` | — | Empirical/Declared |
| `compiler.ambient::tab_b` | fn | `lib/compiler/ambient.myc:243` | `fn tab_b() => Binary{8}` | — | Empirical/Declared |
| `compiler.ambient::cr_b` | fn | `lib/compiler/ambient.myc:244` | `fn cr_b() => Binary{8}` | — | Empirical/Declared |
| `compiler.ambient::nul_b` | fn | `lib/compiler/ambient.myc:245` | `fn nul_b() => Binary{8}` | — | Empirical/Declared |
| `compiler.ambient::escape_string_literal_acc` | fn | `lib/compiler/ambient.myc:254` | `fn escape_string_literal_acc(s: Bytes, i: Binary{32}, len: Binary{32}, acc: Bytes) => Bytes` | escape_string_literal_acc: walks `s` BYTE-by-byte (never char-by-char — this surface has no codepoint-aware iteration), re-escaping the same minimal set `Lexer::lex_string` decodes. Every OTHER byte (including every byte of a multi-byte UTF-8 sequence, which is never equal to any of the ASCII control bytes checked below) is passed through via a 1-byte SLICE of `s` itself — this surface has no "synthesize a Bytes from an arbitrary Binary{8}" primitive, so passthrough must slice the byte from an EXISTING buffer rather than reconstruct it (the same reasoning `digit_bytes` above relies on for the KNOWN 0-9 range). | Empirical/Declared |
| `compiler.ambient::escape_string_literal` | fn | `lib/compiler/ambient.myc:280` | `fn escape_string_literal(s: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::split_lines_acc` | fn | `lib/compiler/ambient.myc:284` | `fn split_lines_acc(s: Bytes, i: Binary{32}, start: Binary{32}, len: Binary{32}, acc: Vec[Bytes]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ambient::split_lines` | fn | `lib/compiler/ambient.myc:297` | `fn split_lines(s: Bytes) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ambient::indent_lines_acc` | fn | `lib/compiler/ambient.myc:300` | `fn indent_lines_acc(prefix: Bytes, lines: Vec[Bytes], acc: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::indent_lines` | fn | `lib/compiler/ambient.myc:308` | `fn indent_lines(prefix: Bytes, text: Bytes) => Bytes` | indent_lines: re-indent every line of `text` by `prefix`, one trailing "\n" each (matches Rust's `for line in text.lines() { s.push_str(&format!("{prefix}{line}\n")); }`). | Empirical/Declared |
| `compiler.ambient::ends_with_nl` | fn | `lib/compiler/ambient.myc:311` | `fn ends_with_nl(s: Bytes) => Bool` | — | Empirical/Declared |
| `compiler.ambient::strip_suffix_nl` | fn | `lib/compiler/ambient.myc:317` | `fn strip_suffix_nl(s: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::trim_end_nl` | fn | `lib/compiler/ambient.myc:323` | `fn trim_end_nl(s: Bytes) => Bytes` | trim_end_nl: mirrors `str::trim_end` as used on `print_impl_decl`'s output inside `print_lower_decl` — that output always ends in exactly one "\n" (never other trailing whitespace), so stripping just that one trailing byte is the faithful equivalent here. | Empirical/Declared |
| `compiler.ambient::join_bytes_list` | fn | `lib/compiler/ambient.myc:327` | `fn join_bytes_list(sep: Bytes, xs: Vec[Bytes]) => Bytes` | join_bytes_list: comma/sep-join a Vec[Bytes] (type-param name lists, `object`/`type` params). | Empirical/Declared |
| `compiler.ambient::AmbientError` | type | `lib/compiler/ambient.myc:337` | `type AmbientError = MultipleDefaults(Paradigm, Paradigm) \| UnresolvedAmbient(Bytes) \| ParadigmShapeMismatch(Bytes, Paradigm, Bytes) \| BareDecimalNoEncoding(Bytes, Paradigm) \| DepthExceeded(Bytes, Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::AmbientError::MultipleDefaults` | ctor | `lib/compiler/ambient.myc:338` | `MultipleDefaults(Paradigm, Paradigm)` | — | Empirical/Declared |
| `compiler.ambient::AmbientError::UnresolvedAmbient` | ctor | `lib/compiler/ambient.myc:339` | `UnresolvedAmbient(Bytes)` | — | Empirical/Declared |
| `compiler.ambient::AmbientError::ParadigmShapeMismatch` | ctor | `lib/compiler/ambient.myc:340` | `ParadigmShapeMismatch(Bytes, Paradigm, Bytes)` | — | Empirical/Declared |
| `compiler.ambient::AmbientError::BareDecimalNoEncoding` | ctor | `lib/compiler/ambient.myc:341` | `BareDecimalNoEncoding(Bytes, Paradigm)` | — | Empirical/Declared |
| `compiler.ambient::AmbientError::DepthExceeded` | ctor | `lib/compiler/ambient.myc:342` | `DepthExceeded(Bytes, Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::ambient_error_kind` | fn | `lib/compiler/ambient.myc:345` | `fn ambient_error_kind(e: AmbientError) => Binary{32}` | ambient_error_kind: a 5-way classification code (FLAG-ambient-3 — message TEXT is not ported). | Empirical/Declared |
| `compiler.ambient::ResolutionNote` | type | `lib/compiler/ambient.myc:356` | `type ResolutionNote = RNote(Bytes, Paradigm, Bytes)` | ResolutionNote / Resolved: kept as real types (a caller can hold/inspect a `Resolved` value); the FLAG-ambient-2 narrowing is that `resolve_report` always returns `notes = Nil` (never populated). | Empirical/Declared |
| `compiler.ambient::ResolutionNote::RNote` | ctor | `lib/compiler/ambient.myc:356` | `RNote(Bytes, Paradigm, Bytes)` | — | Empirical/Declared |
| `compiler.ambient::Resolved` | type | `lib/compiler/ambient.myc:357` | `type Resolved = Rsv(Nodule, Vec[ResolutionNote])` | — | Empirical/Declared |
| `compiler.ambient::Resolved::Rsv` | ctor | `lib/compiler/ambient.myc:357` | `Rsv(Nodule, Vec[ResolutionNote])` | — | Empirical/Declared |
| `compiler.ambient::max_ambient_depth` | fn | `lib/compiler/ambient.myc:360` | `fn max_ambient_depth() => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::enter_depth` | fn | `lib/compiler/ambient.myc:362` | `fn enter_depth(site: Bytes, depth: Binary{32}) => Result[Binary{32}, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::mismatch_detail_size` | fn | `lib/compiler/ambient.myc:370` | `fn mismatch_detail_size() => Bytes` | — | Empirical/Declared |
| `compiler.ambient::mismatch_detail_dense` | fn | `lib/compiler/ambient.myc:371` | `fn mismatch_detail_dense() => Bytes` | — | Empirical/Declared |
| `compiler.ambient::mismatch_detail_vsa` | fn | `lib/compiler/ambient.myc:372` | `fn mismatch_detail_vsa() => Bytes` | — | Empirical/Declared |
| `compiler.ambient::fill_repr` | fn | `lib/compiler/ambient.myc:374` | `fn fill_repr(site: Bytes, p: Paradigm, params: AmbientParams) => Result[BaseType, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_type_ref` | fn | `lib/compiler/ambient.myc:395` | `fn resolve_type_ref(amb: Option[Paradigm], site: Bytes, t: TypeRef, depth: Binary{32}) => Result[TypeRef, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_base_type` | fn | `lib/compiler/ambient.myc:406` | `fn resolve_base_type(amb: Option[Paradigm], site: Bytes, b: BaseType, depth: Binary{32}) => Result[BaseType, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_typeref_list_acc` | fn | `lib/compiler/ambient.myc:426` | `fn resolve_typeref_list_acc(amb: Option[Paradigm], site: Bytes, xs: Vec[TypeRef], depth: Binary{32}, acc: Vec[TypeRef]) => Result[Vec[TypeRef], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_typeref_list` | fn | `lib/compiler/ambient.myc:435` | `fn resolve_typeref_list(amb: Option[Paradigm], site: Bytes, xs: Vec[TypeRef], depth: Binary{32}) => Result[Vec[TypeRef], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_typeref_opt` | fn | `lib/compiler/ambient.myc:438` | `fn resolve_typeref_opt(amb: Option[Paradigm], site: Bytes, t: Option[TypeRef], depth: Binary{32}) => Result[Option[TypeRef], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_literal` | fn | `lib/compiler/ambient.myc:445` | `fn resolve_literal(amb: Option[Paradigm], site: Bytes, l: Literal, depth: Binary{32}) => Result[Literal, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_pattern` | fn | `lib/compiler/ambient.myc:463` | `fn resolve_pattern(amb: Option[Paradigm], site: Bytes, p: Pattern, depth: Binary{32}) => Result[Pattern, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_pattern_list_acc` | fn | `lib/compiler/ambient.myc:476` | `fn resolve_pattern_list_acc(amb: Option[Paradigm], site: Bytes, xs: Vec[Pattern], depth: Binary{32}, acc: Vec[Pattern]) => Result[Vec[Pattern], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_pattern_list` | fn | `lib/compiler/ambient.myc:485` | `fn resolve_pattern_list(amb: Option[Paradigm], site: Bytes, xs: Vec[Pattern], depth: Binary{32}) => Result[Vec[Pattern], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_expr` | fn | `lib/compiler/ambient.myc:489` | `fn resolve_expr(amb: Option[Paradigm], site: Bytes, e: Expr, depth: Binary{32}) => Result[Expr, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_expr_list_acc` | fn | `lib/compiler/ambient.myc:579` | `fn resolve_expr_list_acc(amb: Option[Paradigm], site: Bytes, xs: Vec[Expr], depth: Binary{32}, acc: Vec[Expr]) => Result[Vec[Expr], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_expr_list` | fn | `lib/compiler/ambient.myc:588` | `fn resolve_expr_list(amb: Option[Paradigm], site: Bytes, xs: Vec[Expr], depth: Binary{32}) => Result[Vec[Expr], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_expr_opt` | fn | `lib/compiler/ambient.myc:591` | `fn resolve_expr_opt(amb: Option[Paradigm], site: Bytes, e: Option[Expr], depth: Binary{32}) => Result[Option[Expr], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_arm` | fn | `lib/compiler/ambient.myc:597` | `fn resolve_arm(amb: Option[Paradigm], site: Bytes, a: Arm, depth: Binary{32}) => Result[Arm, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_arm_list_acc` | fn | `lib/compiler/ambient.myc:608` | `fn resolve_arm_list_acc(amb: Option[Paradigm], site: Bytes, xs: Vec[Arm], depth: Binary{32}, acc: Vec[Arm]) => Result[Vec[Arm], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_arm_list` | fn | `lib/compiler/ambient.myc:617` | `fn resolve_arm_list(amb: Option[Paradigm], site: Bytes, xs: Vec[Arm], depth: Binary{32}) => Result[Vec[Arm], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_hypha` | fn | `lib/compiler/ambient.myc:622` | `fn resolve_hypha(amb: Option[Paradigm], site: Bytes, h: Hypha, depth: Binary{32}) => Result[Hypha, AmbientError]` | resolve_hypha: M-906 (DN-70 D1) — a hypha's optional `@forage(policy)` resolves under the same ambient as its body (no new ambient frame). | Empirical/Declared |
| `compiler.ambient::resolve_hypha_list_acc` | fn | `lib/compiler/ambient.myc:633` | `fn resolve_hypha_list_acc(amb: Option[Paradigm], site: Bytes, xs: Vec[Hypha], depth: Binary{32}, acc: Vec[Hypha]) => Result[Vec[Hypha], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_hypha_list` | fn | `lib/compiler/ambient.myc:642` | `fn resolve_hypha_list(amb: Option[Paradigm], site: Bytes, xs: Vec[Hypha], depth: Binary{32}) => Result[Vec[Hypha], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_param` | fn | `lib/compiler/ambient.myc:646` | `fn resolve_param(amb: Option[Paradigm], site: Bytes, p: Param) => Result[Param, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_param_list_acc` | fn | `lib/compiler/ambient.myc:654` | `fn resolve_param_list_acc(amb: Option[Paradigm], site: Bytes, xs: Vec[Param], acc: Vec[Param]) => Result[Vec[Param], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_param_list` | fn | `lib/compiler/ambient.myc:663` | `fn resolve_param_list(amb: Option[Paradigm], site: Bytes, xs: Vec[Param]) => Result[Vec[Param], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_fn_sig` | fn | `lib/compiler/ambient.myc:666` | `fn resolve_fn_sig(amb: Option[Paradigm], s: FnSig) => Result[FnSig, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_fnsig_list_acc` | fn | `lib/compiler/ambient.myc:677` | `fn resolve_fnsig_list_acc(amb: Option[Paradigm], xs: Vec[FnSig], acc: Vec[FnSig]) => Result[Vec[FnSig], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_fnsig_list` | fn | `lib/compiler/ambient.myc:686` | `fn resolve_fnsig_list(amb: Option[Paradigm], xs: Vec[FnSig]) => Result[Vec[FnSig], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_ctor` | fn | `lib/compiler/ambient.myc:690` | `fn resolve_ctor(amb: Option[Paradigm], site: Bytes, c: Ctor) => Result[Ctor, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_ctor_list_acc` | fn | `lib/compiler/ambient.myc:698` | `fn resolve_ctor_list_acc(amb: Option[Paradigm], site: Bytes, xs: Vec[Ctor], acc: Vec[Ctor]) => Result[Vec[Ctor], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_ctor_list` | fn | `lib/compiler/ambient.myc:707` | `fn resolve_ctor_list(amb: Option[Paradigm], site: Bytes, xs: Vec[Ctor]) => Result[Vec[Ctor], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_type_decl` | fn | `lib/compiler/ambient.myc:710` | `fn resolve_type_decl(amb: Option[Paradigm], td: TypeDecl) => Result[TypeDecl, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_trait_decl` | fn | `lib/compiler/ambient.myc:718` | `fn resolve_trait_decl(amb: Option[Paradigm], td: TraitDecl) => Result[TraitDecl, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_fndecl` | fn | `lib/compiler/ambient.myc:727` | `fn resolve_fndecl(amb: Option[Paradigm], f: FnDecl) => Result[FnDecl, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_fndecl_list_acc` | fn | `lib/compiler/ambient.myc:740` | `fn resolve_fndecl_list_acc(amb: Option[Paradigm], xs: Vec[FnDecl], acc: Vec[FnDecl]) => Result[Vec[FnDecl], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_fndecl_list` | fn | `lib/compiler/ambient.myc:749` | `fn resolve_fndecl_list(amb: Option[Paradigm], xs: Vec[FnDecl]) => Result[Vec[FnDecl], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_impl_decl` | fn | `lib/compiler/ambient.myc:752` | `fn resolve_impl_decl(amb: Option[Paradigm], id: ImplDecl) => Result[ImplDecl, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_impldecl_list_acc` | fn | `lib/compiler/ambient.myc:766` | `fn resolve_impldecl_list_acc(amb: Option[Paradigm], xs: Vec[ImplDecl], acc: Vec[ImplDecl]) => Result[Vec[ImplDecl], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_impldecl_list` | fn | `lib/compiler/ambient.myc:775` | `fn resolve_impldecl_list(amb: Option[Paradigm], xs: Vec[ImplDecl]) => Result[Vec[ImplDecl], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_inherent_impl_decl` | fn | `lib/compiler/ambient.myc:778` | `fn resolve_inherent_impl_decl(amb: Option[Paradigm], i: InherentImplDecl) => Result[InherentImplDecl, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_viadecl` | fn | `lib/compiler/ambient.myc:789` | `fn resolve_viadecl(amb: Option[Paradigm], site: Bytes, v: ViaDecl) => Result[ViaDecl, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_viadecl_list_acc` | fn | `lib/compiler/ambient.myc:797` | `fn resolve_viadecl_list_acc(amb: Option[Paradigm], site: Bytes, xs: Vec[ViaDecl], acc: Vec[ViaDecl]) => Result[Vec[ViaDecl], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_viadecl_list` | fn | `lib/compiler/ambient.myc:806` | `fn resolve_viadecl_list(amb: Option[Paradigm], site: Bytes, xs: Vec[ViaDecl]) => Result[Vec[ViaDecl], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_object_decl` | fn | `lib/compiler/ambient.myc:809` | `fn resolve_object_decl(amb: Option[Paradigm], od: ObjectDecl) => Result[ObjectDecl, AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_item` | fn | `lib/compiler/ambient.myc:827` | `fn resolve_item(default_p: Option[Paradigm], i: Item) => Result[Option[Item], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_item_list_acc` | fn | `lib/compiler/ambient.myc:843` | `fn resolve_item_list_acc(default_p: Option[Paradigm], xs: Vec[Item], acc: Vec[Item]) => Result[Vec[Item], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_item_list` | fn | `lib/compiler/ambient.myc:855` | `fn resolve_item_list(default_p: Option[Paradigm], xs: Vec[Item]) => Result[Vec[Item], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::scan_default_acc` | fn | `lib/compiler/ambient.myc:859` | `fn scan_default_acc(xs: Vec[Item], found: Option[Paradigm]) => Result[Option[Paradigm], AmbientError]` | scan_default: the nodule-scope ambient scan — at most one `default paradigm`, else MultipleDefaults. | Empirical/Declared |
| `compiler.ambient::scan_default` | fn | `lib/compiler/ambient.myc:871` | `fn scan_default(xs: Vec[Item]) => Result[Option[Paradigm], AmbientError]` | — | Empirical/Declared |
| `compiler.ambient::resolve_report` | fn | `lib/compiler/ambient.myc:875` | `fn resolve_report(n: Nodule) => Result[Resolved, AmbientError]` | resolve_report: like `resolve`, but also returns the (FLAG-ambient-2: always-empty) notes trace. | Empirical/Declared |
| `compiler.ambient::resolve` | fn | `lib/compiler/ambient.myc:889` | `fn resolve(n: Nodule) => Result[Nodule, AmbientError]` | resolve: rewrite a parsed Nodule to its longhand twin (RFC-0012 §4.3/§4.4). Identity on a program that uses no ambient. | Empirical/Declared |
| `compiler.ambient::path_str` | fn | `lib/compiler/ambient.myc:896` | `fn path_str(p: Path) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::pub_str` | fn | `lib/compiler/ambient.myc:899` | `fn pub_str(v: Vis) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::thaw_str` | fn | `lib/compiler/ambient.myc:902` | `fn thaw_str(t: Bool) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::scalar_str` | fn | `lib/compiler/ambient.myc:905` | `fn scalar_str(s: Scalar) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::paradigm_str` | fn | `lib/compiler/ambient.myc:908` | `fn paradigm_str(p: Paradigm) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::strength_dbg` | fn | `lib/compiler/ambient.myc:911` | `fn strength_dbg(g: Strength) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::sparsity_str` | fn | `lib/compiler/ambient.myc:914` | `fn sparsity_str(s: Sparsity) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::ambient_params_str` | fn | `lib/compiler/ambient.myc:917` | `fn ambient_params_str(p: AmbientParams) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_widthref` | fn | `lib/compiler/ambient.myc:924` | `fn print_widthref(w: WidthRef) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_type_ref_list` | fn | `lib/compiler/ambient.myc:927` | `fn print_type_ref_list(xs: Vec[TypeRef]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_base_type` | fn | `lib/compiler/ambient.myc:930` | `fn print_base_type(b: BaseType) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_type_ref` | fn | `lib/compiler/ambient.myc:952` | `fn print_type_ref(t: TypeRef) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_traitref_list` | fn | `lib/compiler/ambient.myc:958` | `fn print_traitref_list(xs: Vec[TraitRef]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_trait_ref` | fn | `lib/compiler/ambient.myc:961` | `fn print_trait_ref(t: TraitRef) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_type_param` | fn | `lib/compiler/ambient.myc:964` | `fn print_type_param(t: TypeParam) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_type_param_list` | fn | `lib/compiler/ambient.myc:967` | `fn print_type_param_list(xs: Vec[TypeParam]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::filter_type_params_acc` | fn | `lib/compiler/ambient.myc:970` | `fn filter_type_params_acc(xs: Vec[TypeParam], acc: Vec[TypeParam]) => Vec[TypeParam]` | — | Empirical/Declared |
| `compiler.ambient::filter_type_params` | fn | `lib/compiler/ambient.myc:979` | `fn filter_type_params(xs: Vec[TypeParam]) => Vec[TypeParam]` | — | Empirical/Declared |
| `compiler.ambient::filter_width_params_acc` | fn | `lib/compiler/ambient.myc:981` | `fn filter_width_params_acc(xs: Vec[TypeParam], acc: Vec[TypeParam]) => Vec[TypeParam]` | — | Empirical/Declared |
| `compiler.ambient::filter_width_params` | fn | `lib/compiler/ambient.myc:990` | `fn filter_width_params(xs: Vec[TypeParam]) => Vec[TypeParam]` | — | Empirical/Declared |
| `compiler.ambient::print_param_list` | fn | `lib/compiler/ambient.myc:992` | `fn print_param_list(xs: Vec[Param]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_sig_tail` | fn | `lib/compiler/ambient.myc:998` | `fn print_sig_tail(s: FnSig) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_params_brackets` | fn | `lib/compiler/ambient.myc:1007` | `fn print_params_brackets(xs: Vec[Bytes]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_typeref_ann` | fn | `lib/compiler/ambient.myc:1011` | `fn print_typeref_ann(ty: Option[TypeRef]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_param_names` | fn | `lib/compiler/ambient.myc:1014` | `fn print_param_names(xs: Vec[Param]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_expr_list` | fn | `lib/compiler/ambient.myc:1020` | `fn print_expr_list(xs: Vec[Expr]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_one_arm` | fn | `lib/compiler/ambient.myc:1023` | `fn print_one_arm(a: Arm) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_arm_list` | fn | `lib/compiler/ambient.myc:1026` | `fn print_arm_list(xs: Vec[Arm]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_one_hypha` | fn | `lib/compiler/ambient.myc:1031` | `fn print_one_hypha(h: Hypha) => Bytes` | print_one_hypha: the ORACLE's own print form drops the `@forage(policy)` annotation from the rendered text (mirrored here verbatim, not a narrowing this port introduces). | Empirical/Declared |
| `compiler.ambient::print_hypha_list` | fn | `lib/compiler/ambient.myc:1034` | `fn print_hypha_list(xs: Vec[Hypha]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_expr` | fn | `lib/compiler/ambient.myc:1037` | `fn print_expr(e: Expr) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_pattern_list` | fn | `lib/compiler/ambient.myc:1059` | `fn print_pattern_list(xs: Vec[Pattern]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_pattern_or_list` | fn | `lib/compiler/ambient.myc:1062` | `fn print_pattern_or_list(xs: Vec[Pattern]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_pattern` | fn | `lib/compiler/ambient.myc:1065` | `fn print_pattern(p: Pattern) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_literal` | fn | `lib/compiler/ambient.myc:1075` | `fn print_literal(l: Literal) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_ctor` | fn | `lib/compiler/ambient.myc:1088` | `fn print_ctor(c: Ctor) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_ctor_list` | fn | `lib/compiler/ambient.myc:1091` | `fn print_ctor_list(xs: Vec[Ctor]) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_type_decl` | fn | `lib/compiler/ambient.myc:1094` | `fn print_type_decl(td: TypeDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_fn_sig_list_lines_acc` | fn | `lib/compiler/ambient.myc:1099` | `fn print_fn_sig_list_lines_acc(xs: Vec[FnSig], acc: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_trait_decl` | fn | `lib/compiler/ambient.myc:1105` | `fn print_trait_decl(td: TraitDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_fn_decl` | fn | `lib/compiler/ambient.myc:1110` | `fn print_fn_decl(f: FnDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::terminate_item` | fn | `lib/compiler/ambient.myc:1117` | `fn terminate_item(item_text: Bytes) => Bytes` | terminate_item: append the mandatory `;` component terminator (DN-57 §3, M-818) — replace a single trailing "\n" with ";\n", else append a bare ";". | Empirical/Declared |
| `compiler.ambient::print_impldecl_methods_acc` | fn | `lib/compiler/ambient.myc:1123` | `fn print_impldecl_methods_acc(xs: Vec[FnDecl], acc: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_impl_decl` | fn | `lib/compiler/ambient.myc:1126` | `fn print_impl_decl(id: ImplDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_inherent_impl_decl` | fn | `lib/compiler/ambient.myc:1132` | `fn print_inherent_impl_decl(i: InherentImplDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_via_decl` | fn | `lib/compiler/ambient.myc:1137` | `fn print_via_decl(v: ViaDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_via_decl_list_acc` | fn | `lib/compiler/ambient.myc:1143` | `fn print_via_decl_list_acc(xs: Vec[ViaDecl], acc: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_impl_decl_list_indented_acc` | fn | `lib/compiler/ambient.myc:1146` | `fn print_impl_decl_list_indented_acc(xs: Vec[ImplDecl], acc: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_fn_decl_list_indented_acc` | fn | `lib/compiler/ambient.myc:1149` | `fn print_fn_decl_list_indented_acc(xs: Vec[FnDecl], acc: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_object_decl` | fn | `lib/compiler/ambient.myc:1152` | `fn print_object_decl(od: ObjectDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_use` | fn | `lib/compiler/ambient.myc:1165` | `fn print_use(u: UsePath) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_lower_decl` | fn | `lib/compiler/ambient.myc:1171` | `fn print_lower_decl(l: LowerDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_derive_decl` | fn | `lib/compiler/ambient.myc:1177` | `fn print_derive_decl(d: DeriveDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_item` | fn | `lib/compiler/ambient.myc:1181` | `fn print_item(i: Item) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_item_list_acc` | fn | `lib/compiler/ambient.myc:1195` | `fn print_item_list_acc(xs: Vec[Item], acc: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::expand_to_source` | fn | `lib/compiler/ambient.myc:1201` | `fn expand_to_source(n: Nodule) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::print_nodule_list_acc` | fn | `lib/compiler/ambient.myc:1210` | `fn print_nodule_list_acc(xs: Vec[Nodule], acc: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::expand_phylum_to_source` | fn | `lib/compiler/ambient.myc:1219` | `fn expand_phylum_to_source(ph: Phylum) => Bytes` | — | Empirical/Declared |
| `compiler.ambient::Fp` | type | `lib/compiler/ambient.myc:1228` | `type Fp = FP(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::Fp::FP` | ctor | `lib/compiler/ambient.myc:1228` | `FP(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.ambient::fp_hash` | fn | `lib/compiler/ambient.myc:1230` | `fn fp_hash(fp: Fp) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::fp_count` | fn | `lib/compiler/ambient.myc:1231` | `fn fp_count(fp: Fp) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::rotl7` | fn | `lib/compiler/ambient.myc:1233` | `fn rotl7(x: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::fp_tag` | fn | `lib/compiler/ambient.myc:1236` | `fn fp_tag(fp: Fp, tag: Binary{32}) => Fp` | — | Empirical/Declared |
| `compiler.ambient::fp_bytes` | fn | `lib/compiler/ambient.myc:1239` | `fn fp_bytes(fp: Fp, b: Bytes) => Fp` | — | Empirical/Declared |
| `compiler.ambient::fp_u32` | fn | `lib/compiler/ambient.myc:1242` | `fn fp_u32(fp: Fp, n: Binary{32}) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_vis` | fn | `lib/compiler/ambient.myc:1245` | `fn walk_vis(fp: Fp, v: Vis) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_paradigm` | fn | `lib/compiler/ambient.myc:1248` | `fn walk_paradigm(fp: Fp, p: Paradigm) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_scalar` | fn | `lib/compiler/ambient.myc:1256` | `fn walk_scalar(fp: Fp, s: Scalar) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_strength` | fn | `lib/compiler/ambient.myc:1264` | `fn walk_strength(fp: Fp, s: Strength) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_sparsity` | fn | `lib/compiler/ambient.myc:1272` | `fn walk_sparsity(fp: Fp, s: Sparsity) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_paramkind` | fn | `lib/compiler/ambient.myc:1278` | `fn walk_paramkind(fp: Fp, k: ParamKind) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_execmode` | fn | `lib/compiler/ambient.myc:1281` | `fn walk_execmode(fp: Fp, e: ExecutionMode) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_widthref` | fn | `lib/compiler/ambient.myc:1284` | `fn walk_widthref(fp: Fp, w: WidthRef) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_path` | fn | `lib/compiler/ambient.myc:1290` | `fn walk_path(fp: Fp, p: Path) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_bytes_list` | fn | `lib/compiler/ambient.myc:1293` | `fn walk_bytes_list(fp: Fp, xs: Vec[Bytes]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_usepath` | fn | `lib/compiler/ambient.myc:1296` | `fn walk_usepath(fp: Fp, u: UsePath) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_bool` | fn | `lib/compiler/ambient.myc:1299` | `fn walk_bool(fp: Fp, b: Bool) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_ambientparams` | fn | `lib/compiler/ambient.myc:1302` | `fn walk_ambientparams(fp: Fp, a: AmbientParams) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_typeref` | fn | `lib/compiler/ambient.myc:1309` | `fn walk_typeref(fp: Fp, t: TypeRef) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_guarantee_opt` | fn | `lib/compiler/ambient.myc:1312` | `fn walk_guarantee_opt(fp: Fp, g: Option[Strength]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_typeref_list` | fn | `lib/compiler/ambient.myc:1315` | `fn walk_typeref_list(fp: Fp, xs: Vec[TypeRef]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_basetype` | fn | `lib/compiler/ambient.myc:1318` | `fn walk_basetype(fp: Fp, b: BaseType) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_traitref` | fn | `lib/compiler/ambient.myc:1334` | `fn walk_traitref(fp: Fp, t: TraitRef) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_traitref_list` | fn | `lib/compiler/ambient.myc:1337` | `fn walk_traitref_list(fp: Fp, xs: Vec[TraitRef]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_typeparam` | fn | `lib/compiler/ambient.myc:1340` | `fn walk_typeparam(fp: Fp, t: TypeParam) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_typeparam_list` | fn | `lib/compiler/ambient.myc:1343` | `fn walk_typeparam_list(fp: Fp, xs: Vec[TypeParam]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_param` | fn | `lib/compiler/ambient.myc:1346` | `fn walk_param(fp: Fp, p: Param) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_param_list` | fn | `lib/compiler/ambient.myc:1349` | `fn walk_param_list(fp: Fp, xs: Vec[Param]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_bytes_list2` | fn | `lib/compiler/ambient.myc:1352` | `fn walk_bytes_list2(fp: Fp, xs: Vec[Bytes]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_effectbudget_list` | fn | `lib/compiler/ambient.myc:1355` | `fn walk_effectbudget_list(fp: Fp, xs: Vec[EffectBudget]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_fnsig` | fn | `lib/compiler/ambient.myc:1358` | `fn walk_fnsig(fp: Fp, s: FnSig) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_ctor` | fn | `lib/compiler/ambient.myc:1371` | `fn walk_ctor(fp: Fp, c: Ctor) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_ctor_list` | fn | `lib/compiler/ambient.myc:1374` | `fn walk_ctor_list(fp: Fp, xs: Vec[Ctor]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_typedecl` | fn | `lib/compiler/ambient.myc:1377` | `fn walk_typedecl(fp: Fp, t: TypeDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_fnsig_list` | fn | `lib/compiler/ambient.myc:1380` | `fn walk_fnsig_list(fp: Fp, xs: Vec[FnSig]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_traitdecl` | fn | `lib/compiler/ambient.myc:1383` | `fn walk_traitdecl(fp: Fp, t: TraitDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_fndecl` | fn | `lib/compiler/ambient.myc:1386` | `fn walk_fndecl(fp: Fp, f: FnDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_tier_opt` | fn | `lib/compiler/ambient.myc:1391` | `fn walk_tier_opt(fp: Fp, t: Option[ExecutionMode]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_fndecl_list` | fn | `lib/compiler/ambient.myc:1394` | `fn walk_fndecl_list(fp: Fp, xs: Vec[FnDecl]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_impldecl` | fn | `lib/compiler/ambient.myc:1397` | `fn walk_impldecl(fp: Fp, i: ImplDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_viadecl` | fn | `lib/compiler/ambient.myc:1402` | `fn walk_viadecl(fp: Fp, v: ViaDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_viadecl_list` | fn | `lib/compiler/ambient.myc:1405` | `fn walk_viadecl_list(fp: Fp, xs: Vec[ViaDecl]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_impldecl_list` | fn | `lib/compiler/ambient.myc:1408` | `fn walk_impldecl_list(fp: Fp, xs: Vec[ImplDecl]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_objectdecl` | fn | `lib/compiler/ambient.myc:1411` | `fn walk_objectdecl(fp: Fp, o: ObjectDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_inherentimpldecl` | fn | `lib/compiler/ambient.myc:1422` | `fn walk_inherentimpldecl(fp: Fp, i: InherentImplDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_lowerrhs` | fn | `lib/compiler/ambient.myc:1425` | `fn walk_lowerrhs(fp: Fp, r: LowerRhs) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_lowerdecl` | fn | `lib/compiler/ambient.myc:1431` | `fn walk_lowerdecl(fp: Fp, l: LowerDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_derivedecl` | fn | `lib/compiler/ambient.myc:1434` | `fn walk_derivedecl(fp: Fp, d: DeriveDecl) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_expr_list` | fn | `lib/compiler/ambient.myc:1437` | `fn walk_expr_list(fp: Fp, xs: Vec[Expr]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_literal` | fn | `lib/compiler/ambient.myc:1440` | `fn walk_literal(fp: Fp, l: Literal) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_pattern` | fn | `lib/compiler/ambient.myc:1452` | `fn walk_pattern(fp: Fp, p: Pattern) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_pattern_list` | fn | `lib/compiler/ambient.myc:1462` | `fn walk_pattern_list(fp: Fp, xs: Vec[Pattern]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_arm` | fn | `lib/compiler/ambient.myc:1465` | `fn walk_arm(fp: Fp, a: Arm) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_arm_list` | fn | `lib/compiler/ambient.myc:1468` | `fn walk_arm_list(fp: Fp, xs: Vec[Arm]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_expr_opt` | fn | `lib/compiler/ambient.myc:1471` | `fn walk_expr_opt(fp: Fp, e: Option[Expr]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_hypha` | fn | `lib/compiler/ambient.myc:1474` | `fn walk_hypha(fp: Fp, h: Hypha) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_hypha_list` | fn | `lib/compiler/ambient.myc:1477` | `fn walk_hypha_list(fp: Fp, xs: Vec[Hypha]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_typeref_opt` | fn | `lib/compiler/ambient.myc:1480` | `fn walk_typeref_opt(fp: Fp, t: Option[TypeRef]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_expr` | fn | `lib/compiler/ambient.myc:1483` | `fn walk_expr(fp: Fp, e: Expr) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_item` | fn | `lib/compiler/ambient.myc:1505` | `fn walk_item(fp: Fp, i: Item) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_item_list` | fn | `lib/compiler/ambient.myc:1519` | `fn walk_item_list(fp: Fp, xs: Vec[Item]) => Fp` | — | Empirical/Declared |
| `compiler.ambient::walk_nodule` | fn | `lib/compiler/ambient.myc:1522` | `fn walk_nodule(fp: Fp, n: Nodule) => Fp` | — | Empirical/Declared |
| `compiler.ambient::fingerprint_nodule` | fn | `lib/compiler/ambient.myc:1525` | `fn fingerprint_nodule(n: Nodule) => Fp` | — | Empirical/Declared |
| `compiler.ambient::stage4_verdict_fp` | fn | `lib/compiler/ambient.myc:1529` | `fn stage4_verdict_fp(fp: Fp, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::resolve_err_kind` | fn | `lib/compiler/ambient.myc:1536` | `fn resolve_err_kind(n: Nodule) => Binary{32}` | resolve_err_kind: Ok => 0; Err(e) => the 5-way classification code (ambient_error_kind). | Empirical/Declared |
| `compiler.ambient::test_input_1` | fn | `lib/compiler/ambient.myc:1544` | `fn test_input_1() => Nodule` | TC1: pure passthrough — no ambient anywhere; `resolve` must be the identity. | Empirical/Declared |
| `compiler.ambient::test_input_2` | fn | `lib/compiler/ambient.myc:1552` | `fn test_input_2() => Nodule` | TC2: `default paradigm Binary` + a paradigm-less `{8}` return type resolves to `Binary{8}`. | Empirical/Declared |
| `compiler.ambient::test_input_3` | fn | `lib/compiler/ambient.myc:1561` | `fn test_input_3() => Nodule` | TC3: `default paradigm Ternary`, same shape as TC2 but Ternary. | Empirical/Declared |
| `compiler.ambient::test_input_4` | fn | `lib/compiler/ambient.myc:1570` | `fn test_input_4() => Nodule` | TC4: `default paradigm Dense` + `{4, F32}` resolves to `Dense{4, F32}`. | Empirical/Declared |
| `compiler.ambient::test_input_5` | fn | `lib/compiler/ambient.myc:1579` | `fn test_input_5() => Nodule` | TC5: `default paradigm VSA` + `{"hrr", 128, Dense}` resolves to `VSA{"hrr", 128, Dense}`. | Empirical/Declared |
| `compiler.ambient::test_input_6` | fn | `lib/compiler/ambient.myc:1588` | `fn test_input_6() => Nodule` | TC6: nested `with paradigm Ternary { … }` locally overrides a `Binary` nodule default. | Empirical/Declared |
| `compiler.ambient::one32_as_64` | fn | `lib/compiler/ambient.myc:1596` | `fn one32_as_64() => Binary{64}` | — | Empirical/Declared |
| `compiler.ambient::test_input_7` | fn | `lib/compiler/ambient.myc:1599` | `fn test_input_7() => Nodule` | TC7: an `object` declaration with a paradigm-less ctor field, resolved under a Binary default. | Empirical/Declared |
| `compiler.ambient::test_input_8` | fn | `lib/compiler/ambient.myc:1609` | `fn test_input_8() => Nodule` | TC8: a mixed-expr body (let/if/match/app/tuple) under a Binary ambient — several bare decimals each resolve to `AmbientInt(Binary, _)`. | Empirical/Declared |
| `compiler.ambient::test_input_err1` | fn | `lib/compiler/ambient.myc:1621` | `fn test_input_err1() => Nodule` | ERR1: two `default paradigm` declarations -> MultipleDefaults. | Empirical/Declared |
| `compiler.ambient::test_input_err2` | fn | `lib/compiler/ambient.myc:1625` | `fn test_input_err2() => Nodule` | ERR2: a paradigm-less `{8}` with no enclosing ambient anywhere -> UnresolvedAmbient. | Empirical/Declared |
| `compiler.ambient::test_input_err3` | fn | `lib/compiler/ambient.myc:1633` | `fn test_input_err3() => Nodule` | ERR3: a Binary ambient but a Dense-shaped `{4, F32}` param -> ParadigmShapeMismatch. | Empirical/Declared |
| `compiler.ambient::test_input_err4` | fn | `lib/compiler/ambient.myc:1642` | `fn test_input_err4() => Nodule` | ERR4: a bare decimal under a Dense ambient -> BareDecimalNoEncoding. | Empirical/Declared |
| `compiler.ambient::stage4_verdict_1` | fn | `lib/compiler/ambient.myc:1651` | `fn stage4_verdict_1(want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_verdict_2` | fn | `lib/compiler/ambient.myc:1657` | `fn stage4_verdict_2(want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_verdict_3` | fn | `lib/compiler/ambient.myc:1663` | `fn stage4_verdict_3(want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_verdict_4` | fn | `lib/compiler/ambient.myc:1669` | `fn stage4_verdict_4(want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_verdict_5` | fn | `lib/compiler/ambient.myc:1675` | `fn stage4_verdict_5(want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_verdict_6` | fn | `lib/compiler/ambient.myc:1681` | `fn stage4_verdict_6(want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_verdict_7` | fn | `lib/compiler/ambient.myc:1687` | `fn stage4_verdict_7(want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_verdict_8` | fn | `lib/compiler/ambient.myc:1693` | `fn stage4_verdict_8(want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_expand_verdict_1` | fn | `lib/compiler/ambient.myc:1702` | `fn stage4_expand_verdict_1(want_expand: Bytes) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_expand_verdict_2` | fn | `lib/compiler/ambient.myc:1705` | `fn stage4_expand_verdict_2(want_expand: Bytes) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_expand_verdict_resolved_2` | fn | `lib/compiler/ambient.myc:1708` | `fn stage4_expand_verdict_resolved_2(want_expand: Bytes) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_expand_verdict_7` | fn | `lib/compiler/ambient.myc:1714` | `fn stage4_expand_verdict_7(want_expand: Bytes) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_expand_verdict_resolved_7` | fn | `lib/compiler/ambient.myc:1717` | `fn stage4_expand_verdict_resolved_7(want_expand: Bytes) => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_err_kind_1` | fn | `lib/compiler/ambient.myc:1724` | `fn stage4_err_kind_1() => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_err_kind_2` | fn | `lib/compiler/ambient.myc:1725` | `fn stage4_err_kind_2() => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_err_kind_3` | fn | `lib/compiler/ambient.myc:1726` | `fn stage4_err_kind_3() => Binary{32}` | — | Empirical/Declared |
| `compiler.ambient::stage4_err_kind_4` | fn | `lib/compiler/ambient.myc:1727` | `fn stage4_err_kind_4() => Binary{32}` | — | Empirical/Declared |

### compiler.ast

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.ast` | nodule | `lib/compiler/ast.myc:11` | `nodule compiler.ast` | Self-hosted L1 surface-AST data types (M-740 Stage 3a; DN-26 §7.3 row 3). A faithful port of crates/mycelium-l1/src/ast.rs's data-type vocabulary (RFC-0006 §3; DN-02) — pure types + a handful of small helper impls, no upward deps (DN-26 §7.1: "ast -> (none)"). The parser that PRODUCES these values is `compiler.parse` (sibling nodule, DN-26 §7.3 row 3). | Empirical/Declared |
| `compiler.ast::Option` | type | `lib/compiler/ast.myc:127` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.ast::Option::None` | ctor | `lib/compiler/ast.myc:127` | `None` | — | Empirical/Declared |
| `compiler.ast::Option::Some` | ctor | `lib/compiler/ast.myc:127` | `Some(A)` | — | Empirical/Declared |
| `compiler.ast::Vec` | type | `lib/compiler/ast.myc:128` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.ast::Vec::Cons` | ctor | `lib/compiler/ast.myc:128` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.ast::Vec::Nil` | ctor | `lib/compiler/ast.myc:128` | `Nil` | — | Empirical/Declared |
| `compiler.ast::Vis` | type | `lib/compiler/ast.myc:132` | `type Vis = Private \| Pub` | — | Empirical/Declared |
| `compiler.ast::Vis::Private` | ctor | `lib/compiler/ast.myc:132` | `Private` | — | Empirical/Declared |
| `compiler.ast::Vis::Pub` | ctor | `lib/compiler/ast.myc:132` | `Pub` | — | Empirical/Declared |
| `compiler.ast::vis_is_pub` | fn | `lib/compiler/ast.myc:134` | `fn vis_is_pub(v: Vis) => Bool` | — | Empirical/Declared |
| `compiler.ast::Path` | type | `lib/compiler/ast.myc:138` | `type Path = Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.ast::Path::Pth` | ctor | `lib/compiler/ast.myc:138` | `Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.ast::path_segs` | fn | `lib/compiler/ast.myc:140` | `fn path_segs(p: Path) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ast::UsePath` | type | `lib/compiler/ast.myc:145` | `type UsePath = UP(Path, Bool)` | — | Empirical/Declared |
| `compiler.ast::UsePath::UP` | ctor | `lib/compiler/ast.myc:145` | `UP(Path, Bool)` | — | Empirical/Declared |
| `compiler.ast::usepath_path` | fn | `lib/compiler/ast.myc:147` | `fn usepath_path(u: UsePath) => Path` | — | Empirical/Declared |
| `compiler.ast::usepath_glob` | fn | `lib/compiler/ast.myc:150` | `fn usepath_glob(u: UsePath) => Bool` | — | Empirical/Declared |
| `compiler.ast::Paradigm` | type | `lib/compiler/ast.myc:154` | `type Paradigm = PBinary \| PTernary \| PDense \| PVsa` | — | Empirical/Declared |
| `compiler.ast::Paradigm::PBinary` | ctor | `lib/compiler/ast.myc:154` | `PBinary` | — | Empirical/Declared |
| `compiler.ast::Paradigm::PDense` | ctor | `lib/compiler/ast.myc:154` | `PDense` | — | Empirical/Declared |
| `compiler.ast::Paradigm::PTernary` | ctor | `lib/compiler/ast.myc:154` | `PTernary` | — | Empirical/Declared |
| `compiler.ast::Paradigm::PVsa` | ctor | `lib/compiler/ast.myc:154` | `PVsa` | — | Empirical/Declared |
| `compiler.ast::paradigm_to_bytes` | fn | `lib/compiler/ast.myc:158` | `fn paradigm_to_bytes(p: Paradigm) => Bytes` | paradigm_to_bytes: mirrors `impl Display for Paradigm` (FLAG-ast-7 note: this one IS ported — a trivial fixed 4-string lookup, unlike WidthRef's). | Empirical/Declared |
| `compiler.ast::Scalar` | type | `lib/compiler/ast.myc:167` | `type Scalar = SF16 \| SBf16 \| SF32 \| SF64` | — | Empirical/Declared |
| `compiler.ast::Scalar::SBf16` | ctor | `lib/compiler/ast.myc:167` | `SBf16` | — | Empirical/Declared |
| `compiler.ast::Scalar::SF16` | ctor | `lib/compiler/ast.myc:167` | `SF16` | — | Empirical/Declared |
| `compiler.ast::Scalar::SF32` | ctor | `lib/compiler/ast.myc:167` | `SF32` | — | Empirical/Declared |
| `compiler.ast::Scalar::SF64` | ctor | `lib/compiler/ast.myc:167` | `SF64` | — | Empirical/Declared |
| `compiler.ast::Sparsity` | type | `lib/compiler/ast.myc:170` | `type Sparsity = SpDense \| SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.ast::Sparsity::SpDense` | ctor | `lib/compiler/ast.myc:170` | `SpDense` | — | Empirical/Declared |
| `compiler.ast::Sparsity::SpSparse` | ctor | `lib/compiler/ast.myc:170` | `SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.ast::AmbientParams` | type | `lib/compiler/ast.myc:174` | `type AmbientParams = APSize(Binary{32}) \| APDense(Binary{32}, Scalar) \| APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.ast::AmbientParams::APSize` | ctor | `lib/compiler/ast.myc:175` | `APSize(Binary{32})` | — | Empirical/Declared |
| `compiler.ast::AmbientParams::APDense` | ctor | `lib/compiler/ast.myc:176` | `APDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.ast::AmbientParams::APVsa` | ctor | `lib/compiler/ast.myc:177` | `APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.ast::Strength` | type | `lib/compiler/ast.myc:180` | `type Strength = GExact \| GProven \| GEmpirical \| GDeclared` | — | Empirical/Declared |
| `compiler.ast::Strength::GDeclared` | ctor | `lib/compiler/ast.myc:180` | `GDeclared` | — | Empirical/Declared |
| `compiler.ast::Strength::GEmpirical` | ctor | `lib/compiler/ast.myc:180` | `GEmpirical` | — | Empirical/Declared |
| `compiler.ast::Strength::GExact` | ctor | `lib/compiler/ast.myc:180` | `GExact` | — | Empirical/Declared |
| `compiler.ast::Strength::GProven` | ctor | `lib/compiler/ast.myc:180` | `GProven` | — | Empirical/Declared |
| `compiler.ast::strength_rank` | fn | `lib/compiler/ast.myc:183` | `fn strength_rank(s: Strength) => Binary{8}` | strength_rank: mirrors `Strength::rank` (u8 -> Binary{8}; Declared=0 .. Exact=3). | Empirical/Declared |
| `compiler.ast::strength_meet` | fn | `lib/compiler/ast.myc:192` | `fn strength_meet(a: Strength, b: Strength) => Strength` | strength_meet: mirrors `Strength::meet` — the weaker (less-trusted) of the two grades. | Empirical/Declared |
| `compiler.ast::strength_satisfies` | fn | `lib/compiler/ast.myc:199` | `fn strength_satisfies(actual: Strength, demand: Strength) => Bool` | strength_satisfies: mirrors `Strength::satisfies` — `self.rank() >= demand.rank()`. | Empirical/Declared |
| `compiler.ast::WidthRef` | type | `lib/compiler/ast.myc:206` | `type WidthRef = WLit(Binary{32}) \| WName(Bytes)` | — | Empirical/Declared |
| `compiler.ast::WidthRef::WLit` | ctor | `lib/compiler/ast.myc:206` | `WLit(Binary{32})` | — | Empirical/Declared |
| `compiler.ast::WidthRef::WName` | ctor | `lib/compiler/ast.myc:206` | `WName(Bytes)` | — | Empirical/Declared |
| `compiler.ast::ParamKind` | type | `lib/compiler/ast.myc:212` | `type ParamKind = PkType \| PkWidth` | — | Empirical/Declared |
| `compiler.ast::ParamKind::PkType` | ctor | `lib/compiler/ast.myc:212` | `PkType` | — | Empirical/Declared |
| `compiler.ast::ParamKind::PkWidth` | ctor | `lib/compiler/ast.myc:212` | `PkWidth` | — | Empirical/Declared |
| `compiler.ast::paramkind_eq` | fn | `lib/compiler/ast.myc:214` | `fn paramkind_eq(a: ParamKind, b: ParamKind) => Bool` | — | Empirical/Declared |
| `compiler.ast::TraitRef` | type | `lib/compiler/ast.myc:221` | `type TraitRef = TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ast::TraitRef::TRf` | ctor | `lib/compiler/ast.myc:221` | `TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ast::traitref_name` | fn | `lib/compiler/ast.myc:223` | `fn traitref_name(t: TraitRef) => Bytes` | — | Empirical/Declared |
| `compiler.ast::traitref_args` | fn | `lib/compiler/ast.myc:226` | `fn traitref_args(t: TraitRef) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.ast::TypeParam` | type | `lib/compiler/ast.myc:230` | `type TypeParam = TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.ast::TypeParam::TP` | ctor | `lib/compiler/ast.myc:230` | `TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.ast::typeparam_name` | fn | `lib/compiler/ast.myc:232` | `fn typeparam_name(t: TypeParam) => Bytes` | — | Empirical/Declared |
| `compiler.ast::typeparam_kind` | fn | `lib/compiler/ast.myc:235` | `fn typeparam_kind(t: TypeParam) => ParamKind` | — | Empirical/Declared |
| `compiler.ast::typeparam_bounds` | fn | `lib/compiler/ast.myc:238` | `fn typeparam_bounds(t: TypeParam) => Vec[TraitRef]` | — | Empirical/Declared |
| `compiler.ast::typeparam_names_of_kind` | fn | `lib/compiler/ast.myc:244` | `fn typeparam_names_of_kind(ps: Vec[TypeParam], k: ParamKind) => Vec[Bytes]` | typeparam_names_of_kind: shared filter for fn_sig_param_names / fn_sig_width_param_names below. Non-tail, bounded by the fn's OWN type-parameter count (a handful) — not source length; the same nesting-bounded shape nodule.myc's split_dotted/join_segs use (RFC-0041 §7 W7 amendment 11). | Empirical/Declared |
| `compiler.ast::EffectBudget` | type | `lib/compiler/ast.myc:255` | `type EffectBudget = EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.ast::EffectBudget::EB` | ctor | `lib/compiler/ast.myc:255` | `EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.ast::FnSig` | type | `lib/compiler/ast.myc:260` | `type FnSig = FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.ast::FnSig::FS` | ctor | `lib/compiler/ast.myc:260` | `FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.ast::fnsig_name` | fn | `lib/compiler/ast.myc:262` | `fn fnsig_name(s: FnSig) => Bytes` | — | Empirical/Declared |
| `compiler.ast::fnsig_params` | fn | `lib/compiler/ast.myc:265` | `fn fnsig_params(s: FnSig) => Vec[TypeParam]` | — | Empirical/Declared |
| `compiler.ast::fnsig_value_params` | fn | `lib/compiler/ast.myc:268` | `fn fnsig_value_params(s: FnSig) => Vec[Param]` | — | Empirical/Declared |
| `compiler.ast::fnsig_ret` | fn | `lib/compiler/ast.myc:271` | `fn fnsig_ret(s: FnSig) => TypeRef` | — | Empirical/Declared |
| `compiler.ast::fnsig_effects` | fn | `lib/compiler/ast.myc:274` | `fn fnsig_effects(s: FnSig) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ast::fnsig_effect_budgets` | fn | `lib/compiler/ast.myc:277` | `fn fnsig_effect_budgets(s: FnSig) => Vec[EffectBudget]` | — | Empirical/Declared |
| `compiler.ast::fnsig_param_names` | fn | `lib/compiler/ast.myc:281` | `fn fnsig_param_names(s: FnSig) => Vec[Bytes]` | fnsig_param_names: mirrors `FnSig::param_names` (type-kind params only). | Empirical/Declared |
| `compiler.ast::fnsig_width_param_names` | fn | `lib/compiler/ast.myc:285` | `fn fnsig_width_param_names(s: FnSig) => Vec[Bytes]` | fnsig_width_param_names: mirrors `FnSig::width_param_names` (width-kind params only). | Empirical/Declared |
| `compiler.ast::Param` | type | `lib/compiler/ast.myc:289` | `type Param = Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.ast::Param::Prm` | ctor | `lib/compiler/ast.myc:289` | `Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.ast::param_name` | fn | `lib/compiler/ast.myc:291` | `fn param_name(p: Param) => Bytes` | — | Empirical/Declared |
| `compiler.ast::param_ty` | fn | `lib/compiler/ast.myc:294` | `fn param_ty(p: Param) => TypeRef` | — | Empirical/Declared |
| `compiler.ast::TypeRef` | type | `lib/compiler/ast.myc:300` | `type TypeRef = TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.ast::TypeRef::TR` | ctor | `lib/compiler/ast.myc:300` | `TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.ast::typeref_base` | fn | `lib/compiler/ast.myc:302` | `fn typeref_base(t: TypeRef) => BaseType` | — | Empirical/Declared |
| `compiler.ast::typeref_guarantee` | fn | `lib/compiler/ast.myc:305` | `fn typeref_guarantee(t: TypeRef) => Option[Strength]` | — | Empirical/Declared |
| `compiler.ast::typeref_unguaranteed` | fn | `lib/compiler/ast.myc:309` | `fn typeref_unguaranteed(b: BaseType) => TypeRef` | typeref_unguaranteed: mirrors `TypeRef::unguaranteed`. | Empirical/Declared |
| `compiler.ast::typeref_with_guarantee` | fn | `lib/compiler/ast.myc:313` | `fn typeref_with_guarantee(b: BaseType, g: Strength) => TypeRef` | typeref_with_guarantee: mirrors `TypeRef::with_guarantee`. | Empirical/Declared |
| `compiler.ast::BaseType` | type | `lib/compiler/ast.myc:320` | `type BaseType = KwBinary(WidthRef) \| KwTernary(WidthRef) \| KwDense(Binary{32}, Scalar) \| Vsa(Bytes, Binary{32}, Sparsity) \| KwSubstrate(Bytes) \| KwSeq(TypeRef, Binary{32}) \| KwBytes \| KwFloat \| Named(Bytes, Vec[TypeRef]) \| Ambient(AmbientParams) \| FnArrow(TypeRef, TypeRef) \| Tuple(Vec[TypeRef])` | BaseType: mirrors ast.rs::BaseType field-for-field. FLAG-ast-4 renames the 7 repr-keyword collisions (Kw-prefix); FLAG-ast-5 renames `Fn` (cross-type collision with `Item::Fn`) to `FnArrow`. `Vsa`/`Named`/`Ambient`/`Tuple` are bare (no collision after FLAG-ast-4's Paradigm/ AmbientParams renames leave them the sole survivor). | Empirical/Declared |
| `compiler.ast::BaseType::KwBinary` | ctor | `lib/compiler/ast.myc:321` | `KwBinary(WidthRef)` | — | Empirical/Declared |
| `compiler.ast::BaseType::KwTernary` | ctor | `lib/compiler/ast.myc:322` | `KwTernary(WidthRef)` | — | Empirical/Declared |
| `compiler.ast::BaseType::KwDense` | ctor | `lib/compiler/ast.myc:323` | `KwDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.ast::BaseType::Vsa` | ctor | `lib/compiler/ast.myc:324` | `Vsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.ast::BaseType::KwSubstrate` | ctor | `lib/compiler/ast.myc:325` | `KwSubstrate(Bytes)` | — | Empirical/Declared |
| `compiler.ast::BaseType::KwSeq` | ctor | `lib/compiler/ast.myc:326` | `KwSeq(TypeRef, Binary{32})` | — | Empirical/Declared |
| `compiler.ast::BaseType::KwBytes` | ctor | `lib/compiler/ast.myc:327` | `KwBytes` | — | Empirical/Declared |
| `compiler.ast::BaseType::KwFloat` | ctor | `lib/compiler/ast.myc:328` | `KwFloat` | — | Empirical/Declared |
| `compiler.ast::BaseType::Named` | ctor | `lib/compiler/ast.myc:329` | `Named(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ast::BaseType::Ambient` | ctor | `lib/compiler/ast.myc:330` | `Ambient(AmbientParams)` | — | Empirical/Declared |
| `compiler.ast::BaseType::FnArrow` | ctor | `lib/compiler/ast.myc:331` | `FnArrow(TypeRef, TypeRef)` | — | Empirical/Declared |
| `compiler.ast::BaseType::Tuple` | ctor | `lib/compiler/ast.myc:332` | `Tuple(Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ast::ExecutionMode` | type | `lib/compiler/ast.myc:335` | `type ExecutionMode = Interpreted \| Compiled` | — | Empirical/Declared |
| `compiler.ast::ExecutionMode::Compiled` | ctor | `lib/compiler/ast.myc:335` | `Compiled` | — | Empirical/Declared |
| `compiler.ast::ExecutionMode::Interpreted` | ctor | `lib/compiler/ast.myc:335` | `Interpreted` | — | Empirical/Declared |
| `compiler.ast::FnDecl` | type | `lib/compiler/ast.myc:339` | `type FnDecl = FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.ast::FnDecl::FD` | ctor | `lib/compiler/ast.myc:339` | `FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.ast::fndecl_vis` | fn | `lib/compiler/ast.myc:341` | `fn fndecl_vis(f: FnDecl) => Vis` | — | Empirical/Declared |
| `compiler.ast::fndecl_thaw` | fn | `lib/compiler/ast.myc:344` | `fn fndecl_thaw(f: FnDecl) => Bool` | — | Empirical/Declared |
| `compiler.ast::fndecl_tier` | fn | `lib/compiler/ast.myc:347` | `fn fndecl_tier(f: FnDecl) => Option[ExecutionMode]` | — | Empirical/Declared |
| `compiler.ast::fndecl_sig` | fn | `lib/compiler/ast.myc:350` | `fn fndecl_sig(f: FnDecl) => FnSig` | — | Empirical/Declared |
| `compiler.ast::fndecl_body` | fn | `lib/compiler/ast.myc:353` | `fn fndecl_body(f: FnDecl) => Expr` | — | Empirical/Declared |
| `compiler.ast::Ctor` | type | `lib/compiler/ast.myc:357` | `type Ctor = Ctr(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ast::Ctor::Ctr` | ctor | `lib/compiler/ast.myc:357` | `Ctr(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ast::ctor_name` | fn | `lib/compiler/ast.myc:359` | `fn ctor_name(c: Ctor) => Bytes` | — | Empirical/Declared |
| `compiler.ast::ctor_fields` | fn | `lib/compiler/ast.myc:362` | `fn ctor_fields(c: Ctor) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.ast::TypeDecl` | type | `lib/compiler/ast.myc:366` | `type TypeDecl = TD(Vis, Bytes, Vec[Bytes], Vec[Ctor])` | — | Empirical/Declared |
| `compiler.ast::TypeDecl::TD` | ctor | `lib/compiler/ast.myc:366` | `TD(Vis, Bytes, Vec[Bytes], Vec[Ctor])` | — | Empirical/Declared |
| `compiler.ast::typedecl_vis` | fn | `lib/compiler/ast.myc:368` | `fn typedecl_vis(t: TypeDecl) => Vis` | — | Empirical/Declared |
| `compiler.ast::typedecl_name` | fn | `lib/compiler/ast.myc:371` | `fn typedecl_name(t: TypeDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ast::typedecl_params` | fn | `lib/compiler/ast.myc:374` | `fn typedecl_params(t: TypeDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ast::typedecl_ctors` | fn | `lib/compiler/ast.myc:377` | `fn typedecl_ctors(t: TypeDecl) => Vec[Ctor]` | — | Empirical/Declared |
| `compiler.ast::TraitDecl` | type | `lib/compiler/ast.myc:381` | `type TraitDecl = TrD(Vis, Bytes, Vec[Bytes], Vec[FnSig])` | — | Empirical/Declared |
| `compiler.ast::TraitDecl::TrD` | ctor | `lib/compiler/ast.myc:381` | `TrD(Vis, Bytes, Vec[Bytes], Vec[FnSig])` | — | Empirical/Declared |
| `compiler.ast::traitdecl_vis` | fn | `lib/compiler/ast.myc:383` | `fn traitdecl_vis(t: TraitDecl) => Vis` | — | Empirical/Declared |
| `compiler.ast::traitdecl_name` | fn | `lib/compiler/ast.myc:386` | `fn traitdecl_name(t: TraitDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ast::traitdecl_params` | fn | `lib/compiler/ast.myc:389` | `fn traitdecl_params(t: TraitDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ast::traitdecl_sigs` | fn | `lib/compiler/ast.myc:392` | `fn traitdecl_sigs(t: TraitDecl) => Vec[FnSig]` | — | Empirical/Declared |
| `compiler.ast::ImplDecl` | type | `lib/compiler/ast.myc:396` | `type ImplDecl = ImD(Bytes, Vec[TypeRef], TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ast::ImplDecl::ImD` | ctor | `lib/compiler/ast.myc:396` | `ImD(Bytes, Vec[TypeRef], TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ast::impldecl_trait_name` | fn | `lib/compiler/ast.myc:398` | `fn impldecl_trait_name(i: ImplDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ast::impldecl_trait_args` | fn | `lib/compiler/ast.myc:401` | `fn impldecl_trait_args(i: ImplDecl) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.ast::impldecl_for_ty` | fn | `lib/compiler/ast.myc:404` | `fn impldecl_for_ty(i: ImplDecl) => TypeRef` | — | Empirical/Declared |
| `compiler.ast::impldecl_methods` | fn | `lib/compiler/ast.myc:407` | `fn impldecl_methods(i: ImplDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.ast::ViaDecl` | type | `lib/compiler/ast.myc:411` | `type ViaDecl = VD(Binary{32}, Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ast::ViaDecl::VD` | ctor | `lib/compiler/ast.myc:411` | `VD(Binary{32}, Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.ast::viadecl_field_idx` | fn | `lib/compiler/ast.myc:413` | `fn viadecl_field_idx(v: ViaDecl) => Binary{32}` | — | Empirical/Declared |
| `compiler.ast::viadecl_trait_name` | fn | `lib/compiler/ast.myc:416` | `fn viadecl_trait_name(v: ViaDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ast::viadecl_trait_args` | fn | `lib/compiler/ast.myc:419` | `fn viadecl_trait_args(v: ViaDecl) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.ast::ObjectDecl` | type | `lib/compiler/ast.myc:423` | `type ObjectDecl = OD(Vis, Bytes, Vec[Bytes], Ctor, Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ast::ObjectDecl::OD` | ctor | `lib/compiler/ast.myc:423` | `OD(Vis, Bytes, Vec[Bytes], Ctor, Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ast::objectdecl_vis` | fn | `lib/compiler/ast.myc:425` | `fn objectdecl_vis(o: ObjectDecl) => Vis` | — | Empirical/Declared |
| `compiler.ast::objectdecl_name` | fn | `lib/compiler/ast.myc:428` | `fn objectdecl_name(o: ObjectDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ast::objectdecl_params` | fn | `lib/compiler/ast.myc:431` | `fn objectdecl_params(o: ObjectDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ast::objectdecl_ctor` | fn | `lib/compiler/ast.myc:434` | `fn objectdecl_ctor(o: ObjectDecl) => Ctor` | — | Empirical/Declared |
| `compiler.ast::objectdecl_via_decls` | fn | `lib/compiler/ast.myc:437` | `fn objectdecl_via_decls(o: ObjectDecl) => Vec[ViaDecl]` | — | Empirical/Declared |
| `compiler.ast::objectdecl_impls` | fn | `lib/compiler/ast.myc:440` | `fn objectdecl_impls(o: ObjectDecl) => Vec[ImplDecl]` | — | Empirical/Declared |
| `compiler.ast::objectdecl_fns` | fn | `lib/compiler/ast.myc:443` | `fn objectdecl_fns(o: ObjectDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.ast::InherentImplDecl` | type | `lib/compiler/ast.myc:447` | `type InherentImplDecl = IID(TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ast::InherentImplDecl::IID` | ctor | `lib/compiler/ast.myc:447` | `IID(TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.ast::inherentimpldecl_for_ty` | fn | `lib/compiler/ast.myc:449` | `fn inherentimpldecl_for_ty(i: InherentImplDecl) => TypeRef` | — | Empirical/Declared |
| `compiler.ast::inherentimpldecl_methods` | fn | `lib/compiler/ast.myc:452` | `fn inherentimpldecl_methods(i: InherentImplDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.ast::LowerRhs` | type | `lib/compiler/ast.myc:457` | `type LowerRhs = LRExpr(Expr) \| LRImpl(ImplDecl)` | — | Empirical/Declared |
| `compiler.ast::LowerRhs::LRExpr` | ctor | `lib/compiler/ast.myc:457` | `LRExpr(Expr)` | — | Empirical/Declared |
| `compiler.ast::LowerRhs::LRImpl` | ctor | `lib/compiler/ast.myc:457` | `LRImpl(ImplDecl)` | — | Empirical/Declared |
| `compiler.ast::LowerDecl` | type | `lib/compiler/ast.myc:460` | `type LowerDecl = LD(Bytes, Vec[Bytes], LowerRhs)` | — | Empirical/Declared |
| `compiler.ast::LowerDecl::LD` | ctor | `lib/compiler/ast.myc:460` | `LD(Bytes, Vec[Bytes], LowerRhs)` | — | Empirical/Declared |
| `compiler.ast::lowerdecl_name` | fn | `lib/compiler/ast.myc:462` | `fn lowerdecl_name(l: LowerDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ast::lowerdecl_params` | fn | `lib/compiler/ast.myc:465` | `fn lowerdecl_params(l: LowerDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.ast::lowerdecl_rhs` | fn | `lib/compiler/ast.myc:468` | `fn lowerdecl_rhs(l: LowerDecl) => LowerRhs` | — | Empirical/Declared |
| `compiler.ast::lowerdecl_expr_rhs` | fn | `lib/compiler/ast.myc:472` | `fn lowerdecl_expr_rhs(l: LowerDecl) => Option[Expr]` | lowerdecl_expr_rhs: mirrors `LowerDecl::expr_rhs`. | Empirical/Declared |
| `compiler.ast::lowerdecl_impl_rhs` | fn | `lib/compiler/ast.myc:479` | `fn lowerdecl_impl_rhs(l: LowerDecl) => Option[ImplDecl]` | lowerdecl_impl_rhs: mirrors `LowerDecl::impl_rhs`. | Empirical/Declared |
| `compiler.ast::DeriveDecl` | type | `lib/compiler/ast.myc:486` | `type DeriveDecl = DD(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.ast::DeriveDecl::DD` | ctor | `lib/compiler/ast.myc:486` | `DD(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.ast::derivedecl_name` | fn | `lib/compiler/ast.myc:488` | `fn derivedecl_name(d: DeriveDecl) => Bytes` | — | Empirical/Declared |
| `compiler.ast::derivedecl_for_ty` | fn | `lib/compiler/ast.myc:491` | `fn derivedecl_for_ty(d: DeriveDecl) => TypeRef` | — | Empirical/Declared |
| `compiler.ast::Item` | type | `lib/compiler/ast.myc:496` | `type Item = Use(UsePath) \| Default(Paradigm) \| Type(TypeDecl) \| Trait(TraitDecl) \| Impl(ImplDecl) \| Fn(FnDecl) \| Object(ObjectDecl) \| Lower(LowerDecl) \| Derive(DeriveDecl) \| InherentImpl(InherentImplDecl)` | — | Empirical/Declared |
| `compiler.ast::Item::Use` | ctor | `lib/compiler/ast.myc:497` | `Use(UsePath)` | — | Empirical/Declared |
| `compiler.ast::Item::Default` | ctor | `lib/compiler/ast.myc:498` | `Default(Paradigm)` | — | Empirical/Declared |
| `compiler.ast::Item::Type` | ctor | `lib/compiler/ast.myc:499` | `Type(TypeDecl)` | — | Empirical/Declared |
| `compiler.ast::Item::Trait` | ctor | `lib/compiler/ast.myc:500` | `Trait(TraitDecl)` | — | Empirical/Declared |
| `compiler.ast::Item::Impl` | ctor | `lib/compiler/ast.myc:501` | `Impl(ImplDecl)` | — | Empirical/Declared |
| `compiler.ast::Item::Fn` | ctor | `lib/compiler/ast.myc:502` | `Fn(FnDecl)` | — | Empirical/Declared |
| `compiler.ast::Item::Object` | ctor | `lib/compiler/ast.myc:503` | `Object(ObjectDecl)` | — | Empirical/Declared |
| `compiler.ast::Item::Lower` | ctor | `lib/compiler/ast.myc:504` | `Lower(LowerDecl)` | — | Empirical/Declared |
| `compiler.ast::Item::Derive` | ctor | `lib/compiler/ast.myc:505` | `Derive(DeriveDecl)` | — | Empirical/Declared |
| `compiler.ast::Item::InherentImpl` | ctor | `lib/compiler/ast.myc:506` | `InherentImpl(InherentImplDecl)` | — | Empirical/Declared |
| `compiler.ast::Nodule` | type | `lib/compiler/ast.myc:510` | `type Nodule = Nd(Path, Bool, Vec[Item])` | — | Empirical/Declared |
| `compiler.ast::Nodule::Nd` | ctor | `lib/compiler/ast.myc:510` | `Nd(Path, Bool, Vec[Item])` | — | Empirical/Declared |
| `compiler.ast::nodule_path` | fn | `lib/compiler/ast.myc:512` | `fn nodule_path(n: Nodule) => Path` | — | Empirical/Declared |
| `compiler.ast::nodule_std_sys` | fn | `lib/compiler/ast.myc:515` | `fn nodule_std_sys(n: Nodule) => Bool` | — | Empirical/Declared |
| `compiler.ast::nodule_items` | fn | `lib/compiler/ast.myc:518` | `fn nodule_items(n: Nodule) => Vec[Item]` | — | Empirical/Declared |
| `compiler.ast::Phylum` | type | `lib/compiler/ast.myc:522` | `type Phylum = Phy(Option[Path], Vec[Nodule])` | — | Empirical/Declared |
| `compiler.ast::Phylum::Phy` | ctor | `lib/compiler/ast.myc:522` | `Phy(Option[Path], Vec[Nodule])` | — | Empirical/Declared |
| `compiler.ast::phylum_path` | fn | `lib/compiler/ast.myc:524` | `fn phylum_path(p: Phylum) => Option[Path]` | — | Empirical/Declared |
| `compiler.ast::phylum_nodules` | fn | `lib/compiler/ast.myc:527` | `fn phylum_nodules(p: Phylum) => Vec[Nodule]` | — | Empirical/Declared |
| `compiler.ast::phylum_of_one` | fn | `lib/compiler/ast.myc:531` | `fn phylum_of_one(n: Nodule) => Phylum` | phylum_of_one: mirrors `Phylum::of_one` — a phylum-of-one wrapping a single bare nodule. | Empirical/Declared |
| `compiler.ast::Literal` | type | `lib/compiler/ast.myc:537` | `type Literal = Bin(Bytes) \| Trit(Bytes) \| Int(Binary{64}) \| AmbientInt(Paradigm, Binary{64}) \| List(Vec[Expr]) \| LBytes(Bytes) \| Str(Bytes) \| LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.ast::Literal::Bin` | ctor | `lib/compiler/ast.myc:538` | `Bin(Bytes)` | — | Empirical/Declared |
| `compiler.ast::Literal::Trit` | ctor | `lib/compiler/ast.myc:539` | `Trit(Bytes)` | — | Empirical/Declared |
| `compiler.ast::Literal::Int` | ctor | `lib/compiler/ast.myc:540` | `Int(Binary{64})` | — | Empirical/Declared |
| `compiler.ast::Literal::AmbientInt` | ctor | `lib/compiler/ast.myc:541` | `AmbientInt(Paradigm, Binary{64})` | — | Empirical/Declared |
| `compiler.ast::Literal::List` | ctor | `lib/compiler/ast.myc:542` | `List(Vec[Expr])` | — | Empirical/Declared |
| `compiler.ast::Literal::LBytes` | ctor | `lib/compiler/ast.myc:543` | `LBytes(Bytes)` | — | Empirical/Declared |
| `compiler.ast::Literal::Str` | ctor | `lib/compiler/ast.myc:544` | `Str(Bytes)` | — | Empirical/Declared |
| `compiler.ast::Literal::LFloat` | ctor | `lib/compiler/ast.myc:545` | `LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.ast::literal_binary` | fn | `lib/compiler/ast.myc:548` | `fn literal_binary(digits: Bytes) => Literal` | literal_binary: mirrors `Literal::binary`. | Empirical/Declared |
| `compiler.ast::literal_ternary` | fn | `lib/compiler/ast.myc:552` | `fn literal_ternary(trits: Bytes) => Literal` | literal_ternary: mirrors `Literal::ternary`. | Empirical/Declared |
| `compiler.ast::literal_string` | fn | `lib/compiler/ast.myc:556` | `fn literal_string(content: Bytes) => Literal` | literal_string: mirrors `Literal::string`. | Empirical/Declared |
| `compiler.ast::literal_float` | fn | `lib/compiler/ast.myc:560` | `fn literal_float(text: Bytes) => Literal` | literal_float: mirrors `Literal::float`. | Empirical/Declared |
| `compiler.ast::Pattern` | type | `lib/compiler/ast.myc:565` | `type Pattern = PWildcard \| PLit(Literal) \| PCtor(Bytes, Vec[Pattern]) \| PIdent(Bytes) \| PTuple(Vec[Pattern]) \| POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.ast::Pattern::PWildcard` | ctor | `lib/compiler/ast.myc:566` | `PWildcard` | — | Empirical/Declared |
| `compiler.ast::Pattern::PLit` | ctor | `lib/compiler/ast.myc:567` | `PLit(Literal)` | — | Empirical/Declared |
| `compiler.ast::Pattern::PCtor` | ctor | `lib/compiler/ast.myc:568` | `PCtor(Bytes, Vec[Pattern])` | — | Empirical/Declared |
| `compiler.ast::Pattern::PIdent` | ctor | `lib/compiler/ast.myc:569` | `PIdent(Bytes)` | — | Empirical/Declared |
| `compiler.ast::Pattern::PTuple` | ctor | `lib/compiler/ast.myc:570` | `PTuple(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.ast::Pattern::POr` | ctor | `lib/compiler/ast.myc:571` | `POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.ast::Arm` | type | `lib/compiler/ast.myc:574` | `type Arm = Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.ast::Arm::Ar` | ctor | `lib/compiler/ast.myc:574` | `Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.ast::arm_pattern` | fn | `lib/compiler/ast.myc:576` | `fn arm_pattern(a: Arm) => Pattern` | — | Empirical/Declared |
| `compiler.ast::arm_body` | fn | `lib/compiler/ast.myc:579` | `fn arm_body(a: Arm) => Expr` | — | Empirical/Declared |
| `compiler.ast::Hypha` | type | `lib/compiler/ast.myc:584` | `type Hypha = Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.ast::Hypha::Hy` | ctor | `lib/compiler/ast.myc:584` | `Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.ast::hypha_forage` | fn | `lib/compiler/ast.myc:586` | `fn hypha_forage(h: Hypha) => Option[Expr]` | — | Empirical/Declared |
| `compiler.ast::hypha_body` | fn | `lib/compiler/ast.myc:589` | `fn hypha_body(h: Hypha) => Expr` | — | Empirical/Declared |
| `compiler.ast::Expr` | type | `lib/compiler/ast.myc:594` | `type Expr = Let(Bytes, Option[TypeRef], Expr, Expr) \| If(Expr, Expr, Expr) \| Match(Expr, Vec[Arm]) \| For(Bytes, Expr, Bytes, Expr, Expr) \| Swap(Expr, TypeRef, Path) \| WithParadigm(Paradigm, Expr) \| Wild(Expr) \| Spore(Expr) \| Consume(Expr) \| Colony(Vec[Hypha]) \| Lambda(Vec[Param], Expr) \| App(Expr, Vec[Expr]) \| Fuse(Expr, Expr) \| Reclaim(Expr, Expr) \| Path(Path) \| Lit(Literal) \| Ascribe(Expr, TypeRef) \| TupleLit(Vec[Expr])` | — | Empirical/Declared |
| `compiler.ast::Expr::Let` | ctor | `lib/compiler/ast.myc:595` | `Let(Bytes, Option[TypeRef], Expr, Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::If` | ctor | `lib/compiler/ast.myc:596` | `If(Expr, Expr, Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::Match` | ctor | `lib/compiler/ast.myc:597` | `Match(Expr, Vec[Arm])` | — | Empirical/Declared |
| `compiler.ast::Expr::For` | ctor | `lib/compiler/ast.myc:598` | `For(Bytes, Expr, Bytes, Expr, Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::Path` | ctor | `lib/compiler/ast.myc:599` | `Path(Path)` | — | Empirical/Declared |
| `compiler.ast::Expr::Swap` | ctor | `lib/compiler/ast.myc:599` | `Swap(Expr, TypeRef, Path)` | — | Empirical/Declared |
| `compiler.ast::Expr::WithParadigm` | ctor | `lib/compiler/ast.myc:600` | `WithParadigm(Paradigm, Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::Wild` | ctor | `lib/compiler/ast.myc:601` | `Wild(Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::Spore` | ctor | `lib/compiler/ast.myc:602` | `Spore(Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::Consume` | ctor | `lib/compiler/ast.myc:603` | `Consume(Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::Colony` | ctor | `lib/compiler/ast.myc:604` | `Colony(Vec[Hypha])` | — | Empirical/Declared |
| `compiler.ast::Expr::Lambda` | ctor | `lib/compiler/ast.myc:605` | `Lambda(Vec[Param], Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::App` | ctor | `lib/compiler/ast.myc:606` | `App(Expr, Vec[Expr])` | — | Empirical/Declared |
| `compiler.ast::Expr::Fuse` | ctor | `lib/compiler/ast.myc:607` | `Fuse(Expr, Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::Reclaim` | ctor | `lib/compiler/ast.myc:608` | `Reclaim(Expr, Expr)` | — | Empirical/Declared |
| `compiler.ast::Expr::Lit` | ctor | `lib/compiler/ast.myc:610` | `Lit(Literal)` | — | Empirical/Declared |
| `compiler.ast::Expr::Ascribe` | ctor | `lib/compiler/ast.myc:611` | `Ascribe(Expr, TypeRef)` | — | Empirical/Declared |
| `compiler.ast::Expr::TupleLit` | ctor | `lib/compiler/ast.myc:612` | `TupleLit(Vec[Expr])` | — | Empirical/Declared |

### compiler.lex

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.lex` | nodule | `lib/compiler/lex.myc:10` | `nodule compiler.lex` | Self-hosted L1 lexer (M-740 Stage 1; DN-26 §7.3 row 1). A faithful port of crates/mycelium-l1/src/lexer.rs's `lex` entry (the plain, comment-discarding path) over the token/position types this nodule redeclares from `compiler.token` (see FLAG-lex-0). | Empirical/Declared |
| `compiler.lex::Option` | type | `lib/compiler/lex.myc:78` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.lex::Option::None` | ctor | `lib/compiler/lex.myc:78` | `None` | — | Empirical/Declared |
| `compiler.lex::Option::Some` | ctor | `lib/compiler/lex.myc:78` | `Some(A)` | — | Empirical/Declared |
| `compiler.lex::Result` | type | `lib/compiler/lex.myc:81` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `compiler.lex::Result::Err` | ctor | `lib/compiler/lex.myc:81` | `Err(E)` | — | Empirical/Declared |
| `compiler.lex::Result::Ok` | ctor | `lib/compiler/lex.myc:81` | `Ok(A)` | — | Empirical/Declared |
| `compiler.lex::Pair` | type | `lib/compiler/lex.myc:84` | `type Pair[A, B] = Pr(A, B)` | — | Empirical/Declared |
| `compiler.lex::Pair::Pr` | ctor | `lib/compiler/lex.myc:84` | `Pr(A, B)` | — | Empirical/Declared |
| `compiler.lex::Pos` | type | `lib/compiler/lex.myc:87` | `type Pos = P(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.lex::Pos::P` | ctor | `lib/compiler/lex.myc:87` | `P(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.lex::pos_line` | fn | `lib/compiler/lex.myc:89` | `fn pos_line(p: Pos) => Binary{32}` | — | Empirical/Declared |
| `compiler.lex::pos_col` | fn | `lib/compiler/lex.myc:92` | `fn pos_col(p: Pos) => Binary{32}` | — | Empirical/Declared |
| `compiler.lex::ScalarTok` | type | `lib/compiler/lex.myc:96` | `type ScalarTok = SF16 \| SBf16 \| SF32 \| SF64` | — | Empirical/Declared |
| `compiler.lex::ScalarTok::SBf16` | ctor | `lib/compiler/lex.myc:96` | `SBf16` | — | Empirical/Declared |
| `compiler.lex::ScalarTok::SF16` | ctor | `lib/compiler/lex.myc:96` | `SF16` | — | Empirical/Declared |
| `compiler.lex::ScalarTok::SF32` | ctor | `lib/compiler/lex.myc:96` | `SF32` | — | Empirical/Declared |
| `compiler.lex::ScalarTok::SF64` | ctor | `lib/compiler/lex.myc:96` | `SF64` | — | Empirical/Declared |
| `compiler.lex::StrengthTok` | type | `lib/compiler/lex.myc:97` | `type StrengthTok = GExact \| GProven \| GEmpirical \| GDeclared` | — | Empirical/Declared |
| `compiler.lex::StrengthTok::GDeclared` | ctor | `lib/compiler/lex.myc:97` | `GDeclared` | — | Empirical/Declared |
| `compiler.lex::StrengthTok::GEmpirical` | ctor | `lib/compiler/lex.myc:97` | `GEmpirical` | — | Empirical/Declared |
| `compiler.lex::StrengthTok::GExact` | ctor | `lib/compiler/lex.myc:97` | `GExact` | — | Empirical/Declared |
| `compiler.lex::StrengthTok::GProven` | ctor | `lib/compiler/lex.myc:97` | `GProven` | — | Empirical/Declared |
| `compiler.lex::Tok` | type | `lib/compiler/lex.myc:100` | `type Tok = Nodule \| Phylum \| Colony \| Hypha \| Fuse \| Mesh \| Graft \| Cyst \| Xloc \| Forage \| Backbone \| Tier \| Reclaim \| Consume \| Grow \| Derive \| Use \| Pub \| Type \| Trait \| Impl \| Fn \| Matured \| Thaw \| Let \| In \| If \| Then \| Else \| Match \| For \| Swap \| Default \| Paradigm \| With \| Wild \| Spore \| To \| Policy \| Lambda \| Object \| Via \| Lower \| KwBinary \| KwTernary \| KwDense \| Vsa \| BinShort \| TernShort \| EmbShort \| HvecShort \| KwSeq \| KwBytes \| KwFloat \| KwSubstrate \| KwSparse \| Scalar(ScalarTok) \| Strength(StrengthTok) \| Ident(Bytes) \| BinLit(Bytes) \| BytesLit(Bytes) \| TritLit(Bytes) \| StrLit(Bytes) \| Int(Bytes) \| FloatLit(Bytes) \| LParen \| RParen \| LBrace \| RBrace \| LBracket \| RBracket \| LAngle \| RAngle \| Shl \| Shr \| At \| AtStdSys \| Colon \| Comma \| Semi \| Dot \| Pipe \| Plus \| Minus \| Star \| Slash \| Percent \| Caret \| Amp \| AmpAmp \| Eq \| EqEq \| Arrow \| FatArrow \| Bang \| BangEq \| PipePipe \| Eof` | — | Empirical/Declared |
| `compiler.lex::Tok::Colony` | ctor | `lib/compiler/lex.myc:101` | `Colony` | — | Empirical/Declared |
| `compiler.lex::Tok::Nodule` | ctor | `lib/compiler/lex.myc:101` | `Nodule` | — | Empirical/Declared |
| `compiler.lex::Tok::Phylum` | ctor | `lib/compiler/lex.myc:101` | `Phylum` | — | Empirical/Declared |
| `compiler.lex::Tok::Backbone` | ctor | `lib/compiler/lex.myc:102` | `Backbone` | — | Empirical/Declared |
| `compiler.lex::Tok::Cyst` | ctor | `lib/compiler/lex.myc:102` | `Cyst` | — | Empirical/Declared |
| `compiler.lex::Tok::Forage` | ctor | `lib/compiler/lex.myc:102` | `Forage` | — | Empirical/Declared |
| `compiler.lex::Tok::Fuse` | ctor | `lib/compiler/lex.myc:102` | `Fuse` | — | Empirical/Declared |
| `compiler.lex::Tok::Graft` | ctor | `lib/compiler/lex.myc:102` | `Graft` | — | Empirical/Declared |
| `compiler.lex::Tok::Hypha` | ctor | `lib/compiler/lex.myc:102` | `Hypha` | — | Empirical/Declared |
| `compiler.lex::Tok::Mesh` | ctor | `lib/compiler/lex.myc:102` | `Mesh` | — | Empirical/Declared |
| `compiler.lex::Tok::Reclaim` | ctor | `lib/compiler/lex.myc:102` | `Reclaim` | — | Empirical/Declared |
| `compiler.lex::Tok::Tier` | ctor | `lib/compiler/lex.myc:102` | `Tier` | — | Empirical/Declared |
| `compiler.lex::Tok::Xloc` | ctor | `lib/compiler/lex.myc:102` | `Xloc` | — | Empirical/Declared |
| `compiler.lex::Tok::Consume` | ctor | `lib/compiler/lex.myc:103` | `Consume` | — | Empirical/Declared |
| `compiler.lex::Tok::Derive` | ctor | `lib/compiler/lex.myc:103` | `Derive` | — | Empirical/Declared |
| `compiler.lex::Tok::Grow` | ctor | `lib/compiler/lex.myc:103` | `Grow` | — | Empirical/Declared |
| `compiler.lex::Tok::Fn` | ctor | `lib/compiler/lex.myc:104` | `Fn` | — | Empirical/Declared |
| `compiler.lex::Tok::Impl` | ctor | `lib/compiler/lex.myc:104` | `Impl` | — | Empirical/Declared |
| `compiler.lex::Tok::Matured` | ctor | `lib/compiler/lex.myc:104` | `Matured` | — | Empirical/Declared |
| `compiler.lex::Tok::Pub` | ctor | `lib/compiler/lex.myc:104` | `Pub` | — | Empirical/Declared |
| `compiler.lex::Tok::Thaw` | ctor | `lib/compiler/lex.myc:104` | `Thaw` | — | Empirical/Declared |
| `compiler.lex::Tok::Trait` | ctor | `lib/compiler/lex.myc:104` | `Trait` | — | Empirical/Declared |
| `compiler.lex::Tok::Type` | ctor | `lib/compiler/lex.myc:104` | `Type` | — | Empirical/Declared |
| `compiler.lex::Tok::Use` | ctor | `lib/compiler/lex.myc:104` | `Use` | — | Empirical/Declared |
| `compiler.lex::Tok::Else` | ctor | `lib/compiler/lex.myc:105` | `Else` | — | Empirical/Declared |
| `compiler.lex::Tok::For` | ctor | `lib/compiler/lex.myc:105` | `For` | — | Empirical/Declared |
| `compiler.lex::Tok::If` | ctor | `lib/compiler/lex.myc:105` | `If` | — | Empirical/Declared |
| `compiler.lex::Tok::In` | ctor | `lib/compiler/lex.myc:105` | `In` | — | Empirical/Declared |
| `compiler.lex::Tok::Let` | ctor | `lib/compiler/lex.myc:105` | `Let` | — | Empirical/Declared |
| `compiler.lex::Tok::Match` | ctor | `lib/compiler/lex.myc:105` | `Match` | — | Empirical/Declared |
| `compiler.lex::Tok::Swap` | ctor | `lib/compiler/lex.myc:105` | `Swap` | — | Empirical/Declared |
| `compiler.lex::Tok::Then` | ctor | `lib/compiler/lex.myc:105` | `Then` | — | Empirical/Declared |
| `compiler.lex::Tok::Default` | ctor | `lib/compiler/lex.myc:106` | `Default` | — | Empirical/Declared |
| `compiler.lex::Tok::Paradigm` | ctor | `lib/compiler/lex.myc:106` | `Paradigm` | — | Empirical/Declared |
| `compiler.lex::Tok::Policy` | ctor | `lib/compiler/lex.myc:106` | `Policy` | — | Empirical/Declared |
| `compiler.lex::Tok::Spore` | ctor | `lib/compiler/lex.myc:106` | `Spore` | — | Empirical/Declared |
| `compiler.lex::Tok::To` | ctor | `lib/compiler/lex.myc:106` | `To` | — | Empirical/Declared |
| `compiler.lex::Tok::Wild` | ctor | `lib/compiler/lex.myc:106` | `Wild` | — | Empirical/Declared |
| `compiler.lex::Tok::With` | ctor | `lib/compiler/lex.myc:106` | `With` | — | Empirical/Declared |
| `compiler.lex::Tok::Lambda` | ctor | `lib/compiler/lex.myc:107` | `Lambda` | — | Empirical/Declared |
| `compiler.lex::Tok::Lower` | ctor | `lib/compiler/lex.myc:107` | `Lower` | — | Empirical/Declared |
| `compiler.lex::Tok::Object` | ctor | `lib/compiler/lex.myc:107` | `Object` | — | Empirical/Declared |
| `compiler.lex::Tok::Via` | ctor | `lib/compiler/lex.myc:107` | `Via` | — | Empirical/Declared |
| `compiler.lex::Tok::KwBinary` | ctor | `lib/compiler/lex.myc:108` | `KwBinary` | — | Empirical/Declared |
| `compiler.lex::Tok::KwDense` | ctor | `lib/compiler/lex.myc:108` | `KwDense` | — | Empirical/Declared |
| `compiler.lex::Tok::KwTernary` | ctor | `lib/compiler/lex.myc:108` | `KwTernary` | — | Empirical/Declared |
| `compiler.lex::Tok::Vsa` | ctor | `lib/compiler/lex.myc:108` | `Vsa` | — | Empirical/Declared |
| `compiler.lex::Tok::BinShort` | ctor | `lib/compiler/lex.myc:109` | `BinShort` | — | Empirical/Declared |
| `compiler.lex::Tok::EmbShort` | ctor | `lib/compiler/lex.myc:109` | `EmbShort` | — | Empirical/Declared |
| `compiler.lex::Tok::HvecShort` | ctor | `lib/compiler/lex.myc:109` | `HvecShort` | — | Empirical/Declared |
| `compiler.lex::Tok::TernShort` | ctor | `lib/compiler/lex.myc:109` | `TernShort` | — | Empirical/Declared |
| `compiler.lex::Tok::KwBytes` | ctor | `lib/compiler/lex.myc:110` | `KwBytes` | — | Empirical/Declared |
| `compiler.lex::Tok::KwFloat` | ctor | `lib/compiler/lex.myc:110` | `KwFloat` | — | Empirical/Declared |
| `compiler.lex::Tok::KwSeq` | ctor | `lib/compiler/lex.myc:110` | `KwSeq` | — | Empirical/Declared |
| `compiler.lex::Tok::KwSparse` | ctor | `lib/compiler/lex.myc:110` | `KwSparse` | — | Empirical/Declared |
| `compiler.lex::Tok::KwSubstrate` | ctor | `lib/compiler/lex.myc:110` | `KwSubstrate` | — | Empirical/Declared |
| `compiler.lex::Tok::Scalar` | ctor | `lib/compiler/lex.myc:111` | `Scalar(ScalarTok)` | — | Empirical/Declared |
| `compiler.lex::Tok::Strength` | ctor | `lib/compiler/lex.myc:111` | `Strength(StrengthTok)` | — | Empirical/Declared |
| `compiler.lex::Tok::Ident` | ctor | `lib/compiler/lex.myc:112` | `Ident(Bytes)` | — | Empirical/Declared |
| `compiler.lex::Tok::BinLit` | ctor | `lib/compiler/lex.myc:113` | `BinLit(Bytes)` | — | Empirical/Declared |
| `compiler.lex::Tok::BytesLit` | ctor | `lib/compiler/lex.myc:114` | `BytesLit(Bytes)` | — | Empirical/Declared |
| `compiler.lex::Tok::TritLit` | ctor | `lib/compiler/lex.myc:115` | `TritLit(Bytes)` | — | Empirical/Declared |
| `compiler.lex::Tok::StrLit` | ctor | `lib/compiler/lex.myc:116` | `StrLit(Bytes)` | — | Empirical/Declared |
| `compiler.lex::Tok::Int` | ctor | `lib/compiler/lex.myc:117` | `Int(Bytes)` | — | Empirical/Declared |
| `compiler.lex::Tok::FloatLit` | ctor | `lib/compiler/lex.myc:118` | `FloatLit(Bytes)` | — | Empirical/Declared |
| `compiler.lex::Tok::LBrace` | ctor | `lib/compiler/lex.myc:119` | `LBrace` | — | Empirical/Declared |
| `compiler.lex::Tok::LBracket` | ctor | `lib/compiler/lex.myc:119` | `LBracket` | — | Empirical/Declared |
| `compiler.lex::Tok::LParen` | ctor | `lib/compiler/lex.myc:119` | `LParen` | — | Empirical/Declared |
| `compiler.lex::Tok::RBrace` | ctor | `lib/compiler/lex.myc:119` | `RBrace` | — | Empirical/Declared |
| `compiler.lex::Tok::RBracket` | ctor | `lib/compiler/lex.myc:119` | `RBracket` | — | Empirical/Declared |
| `compiler.lex::Tok::RParen` | ctor | `lib/compiler/lex.myc:119` | `RParen` | — | Empirical/Declared |
| `compiler.lex::Tok::LAngle` | ctor | `lib/compiler/lex.myc:120` | `LAngle` | — | Empirical/Declared |
| `compiler.lex::Tok::RAngle` | ctor | `lib/compiler/lex.myc:120` | `RAngle` | — | Empirical/Declared |
| `compiler.lex::Tok::Shl` | ctor | `lib/compiler/lex.myc:120` | `Shl` | — | Empirical/Declared |
| `compiler.lex::Tok::Shr` | ctor | `lib/compiler/lex.myc:120` | `Shr` | — | Empirical/Declared |
| `compiler.lex::Tok::At` | ctor | `lib/compiler/lex.myc:121` | `At` | — | Empirical/Declared |
| `compiler.lex::Tok::AtStdSys` | ctor | `lib/compiler/lex.myc:121` | `AtStdSys` | — | Empirical/Declared |
| `compiler.lex::Tok::Colon` | ctor | `lib/compiler/lex.myc:122` | `Colon` | — | Empirical/Declared |
| `compiler.lex::Tok::Comma` | ctor | `lib/compiler/lex.myc:122` | `Comma` | — | Empirical/Declared |
| `compiler.lex::Tok::Dot` | ctor | `lib/compiler/lex.myc:122` | `Dot` | — | Empirical/Declared |
| `compiler.lex::Tok::Pipe` | ctor | `lib/compiler/lex.myc:122` | `Pipe` | — | Empirical/Declared |
| `compiler.lex::Tok::Semi` | ctor | `lib/compiler/lex.myc:122` | `Semi` | — | Empirical/Declared |
| `compiler.lex::Tok::Amp` | ctor | `lib/compiler/lex.myc:123` | `Amp` | — | Empirical/Declared |
| `compiler.lex::Tok::AmpAmp` | ctor | `lib/compiler/lex.myc:123` | `AmpAmp` | — | Empirical/Declared |
| `compiler.lex::Tok::Caret` | ctor | `lib/compiler/lex.myc:123` | `Caret` | — | Empirical/Declared |
| `compiler.lex::Tok::Minus` | ctor | `lib/compiler/lex.myc:123` | `Minus` | — | Empirical/Declared |
| `compiler.lex::Tok::Percent` | ctor | `lib/compiler/lex.myc:123` | `Percent` | — | Empirical/Declared |
| `compiler.lex::Tok::Plus` | ctor | `lib/compiler/lex.myc:123` | `Plus` | — | Empirical/Declared |
| `compiler.lex::Tok::Slash` | ctor | `lib/compiler/lex.myc:123` | `Slash` | — | Empirical/Declared |
| `compiler.lex::Tok::Star` | ctor | `lib/compiler/lex.myc:123` | `Star` | — | Empirical/Declared |
| `compiler.lex::Tok::Arrow` | ctor | `lib/compiler/lex.myc:124` | `Arrow` | — | Empirical/Declared |
| `compiler.lex::Tok::Bang` | ctor | `lib/compiler/lex.myc:124` | `Bang` | — | Empirical/Declared |
| `compiler.lex::Tok::BangEq` | ctor | `lib/compiler/lex.myc:124` | `BangEq` | — | Empirical/Declared |
| `compiler.lex::Tok::Eq` | ctor | `lib/compiler/lex.myc:124` | `Eq` | — | Empirical/Declared |
| `compiler.lex::Tok::EqEq` | ctor | `lib/compiler/lex.myc:124` | `EqEq` | — | Empirical/Declared |
| `compiler.lex::Tok::FatArrow` | ctor | `lib/compiler/lex.myc:124` | `FatArrow` | — | Empirical/Declared |
| `compiler.lex::Tok::PipePipe` | ctor | `lib/compiler/lex.myc:124` | `PipePipe` | — | Empirical/Declared |
| `compiler.lex::Tok::Eof` | ctor | `lib/compiler/lex.myc:125` | `Eof` | — | Empirical/Declared |
| `compiler.lex::Spanned` | type | `lib/compiler/lex.myc:128` | `type Spanned = Sp(Tok, Pos)` | — | Empirical/Declared |
| `compiler.lex::Spanned::Sp` | ctor | `lib/compiler/lex.myc:128` | `Sp(Tok, Pos)` | — | Empirical/Declared |
| `compiler.lex::sp_tok` | fn | `lib/compiler/lex.myc:130` | `fn sp_tok(s: Spanned) => Tok` | — | Empirical/Declared |
| `compiler.lex::sp_pos` | fn | `lib/compiler/lex.myc:133` | `fn sp_pos(s: Spanned) => Pos` | — | Empirical/Declared |
| `compiler.lex::keyword` | fn | `lib/compiler/lex.myc:140` | `fn keyword(word: Bytes) => Option[Tok]` | — | Empirical/Declared |
| `compiler.lex::Vec` | type | `lib/compiler/lex.myc:403` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.lex::Vec::Cons` | ctor | `lib/compiler/lex.myc:403` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.lex::Vec::Nil` | ctor | `lib/compiler/lex.myc:403` | `Nil` | — | Empirical/Declared |
| `compiler.lex::tok_count` | fn | `lib/compiler/lex.myc:407` | `fn tok_count(v: Vec[Spanned]) => Binary{32}` | tok_count: the number of tokens in a Vec[Spanned] (structural spine-walk; matches only Nil/Cons, never Tok itself — cheap and safe, see FLAG-lex-6). | Empirical/Declared |
| `compiler.lex::nth` | fn | `lib/compiler/lex.myc:416` | `fn nth(v: Vec[Spanned], n: Binary{32}) => Option[Spanned]` | nth: the token at 0-based index `n`, or `None` if out of range. Lets a differential test inspect ONE token's `Tok`/`Pos` at a time without ever matching multiple `Tok` sites in the same nodule check pass (FLAG-lex-6's mitigation). | Empirical/Declared |
| `compiler.lex::LexErr` | type | `lib/compiler/lex.myc:432` | `type LexErr = LE(Pos, Bytes)` | — | Empirical/Declared |
| `compiler.lex::LexErr::LE` | ctor | `lib/compiler/lex.myc:432` | `LE(Pos, Bytes)` | — | Empirical/Declared |
| `compiler.lex::St` | type | `lib/compiler/lex.myc:436` | `type St = S(Binary{32}, Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.lex::St::S` | ctor | `lib/compiler/lex.myc:436` | `S(Binary{32}, Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.lex::st_i` | fn | `lib/compiler/lex.myc:438` | `fn st_i(s: St) => Binary{32}` | — | Empirical/Declared |
| `compiler.lex::st_line` | fn | `lib/compiler/lex.myc:441` | `fn st_line(s: St) => Binary{32}` | — | Empirical/Declared |
| `compiler.lex::st_col` | fn | `lib/compiler/lex.myc:444` | `fn st_col(s: St) => Binary{32}` | — | Empirical/Declared |
| `compiler.lex::st_pos` | fn | `lib/compiler/lex.myc:447` | `fn st_pos(s: St) => Pos` | — | Empirical/Declared |
| `compiler.lex::empty_bytes` | fn | `lib/compiler/lex.myc:454` | `fn empty_bytes() => Bytes` | — | Empirical/Declared |
| `compiler.lex::bool_and` | fn | `lib/compiler/lex.myc:459` | `fn bool_and(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `compiler.lex::bool_or` | fn | `lib/compiler/lex.myc:462` | `fn bool_or(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `compiler.lex::eq_b` | fn | `lib/compiler/lex.myc:465` | `fn eq_b(a: Binary{8}, b: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::lt_b` | fn | `lib/compiler/lex.myc:468` | `fn lt_b(a: Binary{8}, b: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::le_b` | fn | `lib/compiler/lex.myc:472` | `fn le_b(a: Binary{8}, b: Binary{8}) => Bool` | le_b: a <= b, i.e. NOT (b < a). | Empirical/Declared |
| `compiler.lex::in_range` | fn | `lib/compiler/lex.myc:475` | `fn in_range(c: Binary{8}, lo: Binary{8}, hi: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_cont_byte` | fn | `lib/compiler/lex.myc:479` | `fn is_cont_byte(c: Binary{8}) => Bool` | UTF-8 continuation byte: 0x80..0xBF (mirrors lib/std/text.myc::is_cont_byte exactly). | Empirical/Declared |
| `compiler.lex::is_digit` | fn | `lib/compiler/lex.myc:482` | `fn is_digit(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_upper` | fn | `lib/compiler/lex.myc:485` | `fn is_upper(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_lower` | fn | `lib/compiler/lex.myc:488` | `fn is_lower(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_alpha` | fn | `lib/compiler/lex.myc:491` | `fn is_alpha(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_ident_start` | fn | `lib/compiler/lex.myc:494` | `fn is_ident_start(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_ident_continue` | fn | `lib/compiler/lex.myc:497` | `fn is_ident_continue(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_hex_digit` | fn | `lib/compiler/lex.myc:500` | `fn is_hex_digit(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_bin_digit` | fn | `lib/compiler/lex.myc:503` | `fn is_bin_digit(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_trit_glyph` | fn | `lib/compiler/lex.myc:506` | `fn is_trit_glyph(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_underscore` | fn | `lib/compiler/lex.myc:509` | `fn is_underscore(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_ascii_ws` | fn | `lib/compiler/lex.myc:512` | `fn is_ascii_ws(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::is_nl_or_cr` | fn | `lib/compiler/lex.myc:515` | `fn is_nl_or_cr(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.lex::src_peek` | fn | `lib/compiler/lex.myc:519` | `fn src_peek(src: Bytes, i: Binary{32}) => Option[Binary{8}]` | — | Empirical/Declared |
| `compiler.lex::step_pos` | fn | `lib/compiler/lex.myc:524` | `fn step_pos(c: Binary{8}, s: St) => St` | step_pos: the position AFTER consuming byte `c` (FLAG-lex-1: bumps line on '\n', else bumps column only on a non-continuation byte — "count codepoints, not bytes"). | Empirical/Declared |
| `compiler.lex::bump` | fn | `lib/compiler/lex.myc:534` | `fn bump(src: Bytes, s: St) => St` | bump: consume one byte at the current position (never-silent no-op at EOF — total). | Empirical/Declared |
| `compiler.lex::advance_n` | fn | `lib/compiler/lex.myc:538` | `fn advance_n(src: Bytes, s: St, n: Binary{32}) => St` | advance_n: bump `n` times (used after a run-length scan to jump the cursor past a lexeme). | Empirical/Declared |
| `compiler.lex::skip_to_eol` | fn | `lib/compiler/lex.myc:542` | `fn skip_to_eol(src: Bytes, s: St) => St` | — | Empirical/Declared |
| `compiler.lex::skip_trivia` | fn | `lib/compiler/lex.myc:548` | `fn skip_trivia(src: Bytes, s: St) => St` | — | Empirical/Declared |
| `compiler.lex::run_len_ident` | fn | `lib/compiler/lex.myc:567` | `fn run_len_ident(src: Bytes, i: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.lex::run_len_digits` | fn | `lib/compiler/lex.myc:573` | `fn run_len_digits(src: Bytes, i: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.lex::scan_bin_run` | fn | `lib/compiler/lex.myc:580` | `fn scan_bin_run(src: Bytes, i: Binary{32}) => Pair[Binary{32}, Binary{32}]` | scan_bin_run: Pair(total run length INCLUDING `_` separators, count of actual `0`/`1` digits). | Empirical/Declared |
| `compiler.lex::scan_hex_run` | fn | `lib/compiler/lex.myc:593` | `fn scan_hex_run(src: Bytes, i: Binary{32}) => Pair[Binary{32}, Binary{32}]` | scan_hex_run: Pair(total run length INCLUDING `_` separators, count of actual hex digits). | Empirical/Declared |
| `compiler.lex::scan_trit_run` | fn | `lib/compiler/lex.myc:606` | `fn scan_trit_run(src: Bytes, i: Binary{32}) => Binary{32}` | scan_trit_run: total count of trit glyphs (`+`/`0`/`-`; no `_` separator in the trit grammar). | Empirical/Declared |
| `compiler.lex::scan_string_escape` | fn | `lib/compiler/lex.myc:615` | `fn scan_string_escape(src: Bytes, s: St) => Result[Pair[Bytes, St], LexErr]` | — | Empirical/Declared |
| `compiler.lex::scan_string` | fn | `lib/compiler/lex.myc:632` | `fn scan_string(src: Bytes, s: St) => Result[Pair[Bytes, St], LexErr]` | scan_string: `s` is the position right after the opening `"`. Returns the DECODED content + the position right after the closing `"` (never-silent: unterminated / raw newline / bad escape are explicit `Err`, matching lexer.rs::lex_string's contract — see FLAG-lex-7 for message fidelity). | Empirical/Declared |
| `compiler.lex::lex_rangle` | fn | `lib/compiler/lex.myc:661` | `fn lex_rangle(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.lex::lex_langle` | fn | `lib/compiler/lex.myc:668` | `fn lex_langle(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.lex::lex_amp` | fn | `lib/compiler/lex.myc:675` | `fn lex_amp(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.lex::lex_pipe` | fn | `lib/compiler/lex.myc:682` | `fn lex_pipe(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.lex::lex_bang` | fn | `lib/compiler/lex.myc:689` | `fn lex_bang(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.lex::lex_eq` | fn | `lib/compiler/lex.myc:696` | `fn lex_eq(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.lex::lex_dash` | fn | `lib/compiler/lex.myc:704` | `fn lex_dash(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.lex::lex_at` | fn | `lib/compiler/lex.myc:713` | `fn lex_at(src: Bytes, s: St) => Pair[Tok, St]` | `@` vs the atomic `@std-sys` nodule-header marker (M-661; mirrors lexer.rs::lex_at exactly, now expressible directly via the `bytes_eq` prim, M-912). | Empirical/Declared |
| `compiler.lex::lex_ident` | fn | `lib/compiler/lex.myc:731` | `fn lex_ident(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.lex::lex_int` | fn | `lib/compiler/lex.myc:740` | `fn lex_int(src: Bytes, s: St) => Pair[Tok, St]` | lex_int: a non-negative decimal digit run, verbatim (FLAG-lex-2: no float extension, no eager numeric conversion — a structural deviation from lexer.rs, deliberately scoped, no accept-corpus input exercises the gap). | Empirical/Declared |
| `compiler.lex::lex_binlit` | fn | `lib/compiler/lex.myc:745` | `fn lex_binlit(src: Bytes, s: St) => Result[Pair[Tok, St], LexErr]` | — | Empirical/Declared |
| `compiler.lex::lex_hexlit` | fn | `lib/compiler/lex.myc:758` | `fn lex_hexlit(src: Bytes, s: St) => Result[Pair[Tok, St], LexErr]` | Even-hex-digit-count check (RFC-0032 D4: a byte is two hex chars): `and(digits, 1)` masks the low bit — `0` iff `digits` is even. `and` requires same-width operands; the bare `1` anchors to `digits`'s width (Binary{32}, RFC-0012 ambient bare-decimal inference). | Empirical/Declared |
| `compiler.lex::lex_tritlit` | fn | `lib/compiler/lex.myc:770` | `fn lex_tritlit(src: Bytes, s: St) => Result[Pair[Tok, St], LexErr]` | — | Empirical/Declared |
| `compiler.lex::lex_string` | fn | `lib/compiler/lex.myc:778` | `fn lex_string(src: Bytes, s: St) => Result[Pair[Tok, St], LexErr]` | — | Empirical/Declared |
| `compiler.lex::lex_zero_prefixed` | fn | `lib/compiler/lex.myc:785` | `fn lex_zero_prefixed(src: Bytes, s: St) => Result[Pair[Tok, St], LexErr]` | `0` followed by `b`/`x`/`t` opens a base-prefixed literal; otherwise a plain decimal int. | Empirical/Declared |
| `compiler.lex::next_token` | fn | `lib/compiler/lex.myc:797` | `fn next_token(src: Bytes, s: St) => Result[Pair[Tok, St], LexErr]` | — | Empirical/Declared |
| `compiler.lex::run` | fn | `lib/compiler/lex.myc:833` | `fn run(src: Bytes, s: St) => Result[Vec[Spanned], LexErr]` | — | Empirical/Declared |
| `compiler.lex::lex` | fn | `lib/compiler/lex.myc:851` | `fn lex(src: Bytes) => Result[Vec[Spanned], LexErr]` | lex: tokenize `src` into a `Spanned` stream terminated by `Eof` (mirrors lexer.rs::lex). Comments are discarded (FLAG-lex-5); never-silent on any lexically invalid input (an explicit `Err`, never a panic or a silently-skipped character). | Empirical/Declared |

### compiler.nodule_header

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.nodule_header` | nodule | `lib/compiler/nodule.myc:11` | `nodule compiler.nodule_header` | Self-hosted nodule-header recogniser (M-740 Stage 2; DN-26 §7.3 row 2). A faithful port of crates/mycelium-l1/src/nodule.rs — the DN-06 §6 first-non-blank-line `// nodule[: name]` marker recogniser (a source-text concern: comments are lexer trivia, the marker never reaches the AST; metadata is not identity, ADR-003). | Empirical/Declared |
| `compiler.nodule_header::Option` | type | `lib/compiler/nodule.myc:76` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.nodule_header::Option::None` | ctor | `lib/compiler/nodule.myc:76` | `None` | — | Empirical/Declared |
| `compiler.nodule_header::Option::Some` | ctor | `lib/compiler/nodule.myc:76` | `Some(A)` | — | Empirical/Declared |
| `compiler.nodule_header::Result` | type | `lib/compiler/nodule.myc:77` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `compiler.nodule_header::Result::Err` | ctor | `lib/compiler/nodule.myc:77` | `Err(E)` | — | Empirical/Declared |
| `compiler.nodule_header::Result::Ok` | ctor | `lib/compiler/nodule.myc:77` | `Ok(A)` | — | Empirical/Declared |
| `compiler.nodule_header::Vec` | type | `lib/compiler/nodule.myc:78` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.nodule_header::Vec::Cons` | ctor | `lib/compiler/nodule.myc:78` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.nodule_header::Vec::Nil` | ctor | `lib/compiler/nodule.myc:78` | `Nil` | — | Empirical/Declared |
| `compiler.nodule_header::NoduleHeader` | type | `lib/compiler/nodule.myc:83` | `type NoduleHeader = NH(Option[Vec[Bytes]])` | — | Empirical/Declared |
| `compiler.nodule_header::NoduleHeader::NH` | ctor | `lib/compiler/nodule.myc:83` | `NH(Option[Vec[Bytes]])` | — | Empirical/Declared |
| `compiler.nodule_header::nh_name` | fn | `lib/compiler/nodule.myc:85` | `fn nh_name(h: NoduleHeader) => Option[Vec[Bytes]]` | — | Empirical/Declared |
| `compiler.nodule_header::NoduleHeaderError` | type | `lib/compiler/nodule.myc:89` | `type NoduleHeaderError = NHE(Binary{32}, Bytes)` | — | Empirical/Declared |
| `compiler.nodule_header::NoduleHeaderError::NHE` | ctor | `lib/compiler/nodule.myc:89` | `NHE(Binary{32}, Bytes)` | — | Empirical/Declared |
| `compiler.nodule_header::nhe_line` | fn | `lib/compiler/nodule.myc:91` | `fn nhe_line(e: NoduleHeaderError) => Binary{32}` | — | Empirical/Declared |
| `compiler.nodule_header::nhe_message` | fn | `lib/compiler/nodule.myc:94` | `fn nhe_message(e: NoduleHeaderError) => Bytes` | — | Empirical/Declared |
| `compiler.nodule_header::empty_bytes` | fn | `lib/compiler/nodule.myc:98` | `fn empty_bytes() => Bytes` | — | Empirical/Declared |
| `compiler.nodule_header::join_segs` | fn | `lib/compiler/nodule.myc:104` | `fn join_segs(segs: Vec[Bytes]) => Bytes` | — | Empirical/Declared |
| `compiler.nodule_header::dotted` | fn | `lib/compiler/nodule.myc:113` | `fn dotted(h: NoduleHeader) => Option[Bytes]` | — | Empirical/Declared |
| `compiler.nodule_header::canonical` | fn | `lib/compiler/nodule.myc:117` | `fn canonical(h: NoduleHeader) => Bytes` | canonical: the one-line spelling the formatter (M-142) emits. | Empirical/Declared |
| `compiler.nodule_header::bool_or` | fn | `lib/compiler/nodule.myc:124` | `fn bool_or(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `compiler.nodule_header::eq_b` | fn | `lib/compiler/nodule.myc:127` | `fn eq_b(a: Binary{8}, b: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.nodule_header::le_b` | fn | `lib/compiler/nodule.myc:131` | `fn le_b(a: Binary{8}, b: Binary{8}) => Bool` | le_b: a <= b, i.e. NOT (b < a). | Empirical/Declared |
| `compiler.nodule_header::in_range` | fn | `lib/compiler/nodule.myc:134` | `fn in_range(c: Binary{8}, lo: Binary{8}, hi: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.nodule_header::is_digit` | fn | `lib/compiler/nodule.myc:137` | `fn is_digit(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.nodule_header::is_upper` | fn | `lib/compiler/nodule.myc:140` | `fn is_upper(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.nodule_header::is_lower` | fn | `lib/compiler/nodule.myc:143` | `fn is_lower(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.nodule_header::is_alpha` | fn | `lib/compiler/nodule.myc:146` | `fn is_alpha(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.nodule_header::is_ident_start` | fn | `lib/compiler/nodule.myc:150` | `fn is_ident_start(c: Binary{8}) => Bool` | is_ident_start: mirrors nodule.rs::is_ident's first-char rule (ASCII letter or '_'). | Empirical/Declared |
| `compiler.nodule_header::is_ident_continue` | fn | `lib/compiler/nodule.myc:154` | `fn is_ident_continue(c: Binary{8}) => Bool` | is_ident_continue: mirrors nodule.rs::is_ident's rest rule (ASCII alphanumeric or '_'). | Empirical/Declared |
| `compiler.nodule_header::is_ascii_ws` | fn | `lib/compiler/nodule.myc:158` | `fn is_ascii_ws(c: Binary{8}) => Bool` | ASCII whitespace: space / tab / CR / LF (FLAG-nodule-2). | Empirical/Declared |
| `compiler.nodule_header::line_end` | fn | `lib/compiler/nodule.myc:163` | `fn line_end(src: Bytes, i: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.nodule_header::trim_start` | fn | `lib/compiler/nodule.myc:173` | `fn trim_start(src: Bytes, i: Binary{32}, e: Binary{32}) => Binary{32}` | trim_start: the first index in [i, e) whose byte is not ASCII whitespace, or `e`. | Empirical/Declared |
| `compiler.nodule_header::trim_end` | fn | `lib/compiler/nodule.myc:183` | `fn trim_end(src: Bytes, i: Binary{32}, e: Binary{32}) => Binary{32}` | trim_end: the end index of [i, e) with trailing ASCII whitespace dropped (>= i). | Empirical/Declared |
| `compiler.nodule_header::next_dot` | fn | `lib/compiler/nodule.myc:194` | `fn next_dot(name: Bytes, i: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.nodule_header::ident_run_ok` | fn | `lib/compiler/nodule.myc:205` | `fn ident_run_ok(name: Bytes, i: Binary{32}, j: Binary{32}) => Bool` | ident_run_ok: every byte in [i, j) is a valid identifier-continue byte (caller keeps i <= j and j <= bytes_len(name), so bytes_get stays in range). | Empirical/Declared |
| `compiler.nodule_header::seg_is_ident` | fn | `lib/compiler/nodule.myc:216` | `fn seg_is_ident(name: Bytes, i: Binary{32}, j: Binary{32}) => Bool` | seg_is_ident: name[i..j) is a valid identifier (letters, digits, '_'; not starting with a digit). Caller guarantees i < j (the empty segment is refused before this is called). | Empirical/Declared |
| `compiler.nodule_header::split_dotted` | fn | `lib/compiler/nodule.myc:226` | `fn split_dotted(name: Bytes, i: Binary{32}, line_no: Binary{32}) => Result[Vec[Bytes], NoduleHeaderError]` | split_dotted: validate + split `name[i..]` on '.' into segments (mirrors parse_dotted: a never-silent Err for an empty segment — leading/trailing/doubled dots — or a non-identifier segment). Non-tail, bounded by the segment count (TCO note above). Messages are static (FLAG-nodule-3). | Empirical/Declared |
| `compiler.nodule_header::recognise_named` | fn | `lib/compiler/nodule.myc:246` | `fn recognise_named(body: Bytes, line_no: Binary{32}) => Result[Option[NoduleHeader], NoduleHeaderError]` | — | Empirical/Declared |
| `compiler.nodule_header::recognise` | fn | `lib/compiler/nodule.myc:259` | `fn recognise(line: Bytes, line_no: Binary{32}) => Result[Option[NoduleHeader], NoduleHeaderError]` | recognise: classify one TRIMMED, non-empty first-non-blank line. Must be a `//` line comment to be a marker at all; then exactly `nodule` (bare) or `nodule:<name>` (named); anything else is an ordinary comment (Ok(None)) — e.g. `/// nodule` and `// nodule is a word` are NOT markers. | Empirical/Declared |
| `compiler.nodule_header::scan_lines` | fn | `lib/compiler/nodule.myc:283` | `fn scan_lines(src: Bytes, i: Binary{32}, line_no: Binary{32}) => Result[Option[NoduleHeader], NoduleHeaderError]` | — | Empirical/Declared |
| `compiler.nodule_header::parse_nodule_header` | fn | `lib/compiler/nodule.myc:298` | `fn parse_nodule_header(src: Bytes) => Result[Option[NoduleHeader], NoduleHeaderError]` | parse_nodule_header: recognise the optional nodule marker on the first non-blank line of `src`. Ok(Some(h)) — well-formed marker; Ok(None) — not a marker; Err — ill-formed NAMED marker (G2). | Empirical/Declared |

### compiler.parse

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.parse` | nodule | `lib/compiler/parse.myc:14` | `nodule compiler.parse` | Self-hosted L1 recursive-descent parser (M-740 Stage 3b; DN-26 §7.3 row 3). A faithful port of crates/mycelium-l1/src/parse.rs's `parse` entry (source text -> Nodule) over a SELF- CONTAINED copy of the token/lexer types (mirroring compiler.token/compiler.lex) and the AST vocabulary (mirroring compiler.ast) — per M-982, cross-nodule EXECUTION is still staged, so this nodule cannot `use compiler.lex.\*`/`use compiler.ast.\*` and actually RUN them; it redeclares everything needed to go all the way from `source text -> AST` in one nodule, matching crate::parse::parse(src: &str) -> Result<Nodule, ParseError> directly. | Empirical/Declared |
| `compiler.parse::Option` | type | `lib/compiler/parse.myc:161` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.parse::Option::None` | ctor | `lib/compiler/parse.myc:161` | `None` | — | Empirical/Declared |
| `compiler.parse::Option::Some` | ctor | `lib/compiler/parse.myc:161` | `Some(A)` | — | Empirical/Declared |
| `compiler.parse::Result` | type | `lib/compiler/parse.myc:162` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `compiler.parse::Result::Err` | ctor | `lib/compiler/parse.myc:162` | `Err(E)` | — | Empirical/Declared |
| `compiler.parse::Result::Ok` | ctor | `lib/compiler/parse.myc:162` | `Ok(A)` | — | Empirical/Declared |
| `compiler.parse::Vec` | type | `lib/compiler/parse.myc:163` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.parse::Vec::Cons` | ctor | `lib/compiler/parse.myc:163` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.parse::Vec::Nil` | ctor | `lib/compiler/parse.myc:163` | `Nil` | — | Empirical/Declared |
| `compiler.parse::Pair` | type | `lib/compiler/parse.myc:164` | `type Pair[A, B] = Pr(A, B)` | — | Empirical/Declared |
| `compiler.parse::Pair::Pr` | ctor | `lib/compiler/parse.myc:164` | `Pr(A, B)` | — | Empirical/Declared |
| `compiler.parse::rev_acc` | fn | `lib/compiler/parse.myc:173` | `fn rev_acc[A](xs: Vec[A], acc: Vec[A]) => Vec[A]` | rev_acc: the direct-tail list reversal underpinning every accumulator+reverse list-building loop in this nodule (RFC-0041 SS7 W7 amendment 11 -- the TCO acceptance criterion): `rev_acc(xs, acc)` = reverse(xs) ++ acc, one direct tail call per cell. Every loop whose depth is bounded by SOURCE LENGTH (token/item/arm/segment/byte count) accumulates via `loop(ts, Cons(item, acc))` (a direct tail call, so the evaluator's TCO reuses the frame) and restores source order with one final `rev_acc(acc, Nil)` -- O(n) work, O(1) eval stack depth. Expression/type/pattern NESTING recursion is the depth-budgeted class instead (FLAG-parse-7). | Empirical/Declared |
| `compiler.parse::Pos` | type | `lib/compiler/parse.myc:177` | `type Pos = P(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.parse::Pos::P` | ctor | `lib/compiler/parse.myc:177` | `P(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.parse::pos_line` | fn | `lib/compiler/parse.myc:179` | `fn pos_line(p: Pos) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::pos_col` | fn | `lib/compiler/parse.myc:182` | `fn pos_col(p: Pos) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::ScalarTok` | type | `lib/compiler/parse.myc:188` | `type ScalarTok = TSF16 \| TSBf16 \| TSF32 \| TSF64` | — | Empirical/Declared |
| `compiler.parse::ScalarTok::TSBf16` | ctor | `lib/compiler/parse.myc:188` | `TSBf16` | — | Empirical/Declared |
| `compiler.parse::ScalarTok::TSF16` | ctor | `lib/compiler/parse.myc:188` | `TSF16` | — | Empirical/Declared |
| `compiler.parse::ScalarTok::TSF32` | ctor | `lib/compiler/parse.myc:188` | `TSF32` | — | Empirical/Declared |
| `compiler.parse::ScalarTok::TSF64` | ctor | `lib/compiler/parse.myc:188` | `TSF64` | — | Empirical/Declared |
| `compiler.parse::StrengthTok` | type | `lib/compiler/parse.myc:189` | `type StrengthTok = TGExact \| TGProven \| TGEmpirical \| TGDeclared` | — | Empirical/Declared |
| `compiler.parse::StrengthTok::TGDeclared` | ctor | `lib/compiler/parse.myc:189` | `TGDeclared` | — | Empirical/Declared |
| `compiler.parse::StrengthTok::TGEmpirical` | ctor | `lib/compiler/parse.myc:189` | `TGEmpirical` | — | Empirical/Declared |
| `compiler.parse::StrengthTok::TGExact` | ctor | `lib/compiler/parse.myc:189` | `TGExact` | — | Empirical/Declared |
| `compiler.parse::StrengthTok::TGProven` | ctor | `lib/compiler/parse.myc:189` | `TGProven` | — | Empirical/Declared |
| `compiler.parse::Tok` | type | `lib/compiler/parse.myc:193` | `type Tok = Nodule \| Phylum \| TColony \| Hypha \| TFuse \| Mesh \| Graft \| Cyst \| Xloc \| Forage \| Backbone \| Tier \| TReclaim \| TConsume \| Grow \| TDerive \| TUse \| TPub \| TType \| TTrait \| TImpl \| TFn \| Matured \| Thaw \| TLet \| In \| TIf \| Then \| Else \| TMatch \| TFor \| TSwap \| TDefault \| Paradigm \| With \| TWild \| TSpore \| To \| Policy \| TLambda \| TObject \| Via \| TLower \| TKwBinary \| TKwTernary \| TKwDense \| TVsa \| BinShort \| TernShort \| EmbShort \| HvecShort \| TKwSeq \| TKwBytes \| TKwFloat \| TKwSubstrate \| KwSparse \| Scalar(ScalarTok) \| Strength(StrengthTok) \| Ident(Bytes) \| BinLit(Bytes) \| BytesLit(Bytes) \| TritLit(Bytes) \| StrLit(Bytes) \| TInt(Bytes) \| FloatLit(Bytes) \| LParen \| RParen \| LBrace \| RBrace \| LBracket \| RBracket \| LAngle \| RAngle \| Shl \| Shr \| At \| AtStdSys \| Colon \| Comma \| Semi \| Dot \| Pipe \| Plus \| Minus \| Star \| Slash \| Percent \| Caret \| Amp \| AmpAmp \| Eq \| EqEq \| Arrow \| FatArrow \| Bang \| BangEq \| PipePipe \| Eof` | — | Empirical/Declared |
| `compiler.parse::Tok::Nodule` | ctor | `lib/compiler/parse.myc:194` | `Nodule` | — | Empirical/Declared |
| `compiler.parse::Tok::Phylum` | ctor | `lib/compiler/parse.myc:194` | `Phylum` | — | Empirical/Declared |
| `compiler.parse::Tok::TColony` | ctor | `lib/compiler/parse.myc:194` | `TColony` | — | Empirical/Declared |
| `compiler.parse::Tok::Backbone` | ctor | `lib/compiler/parse.myc:195` | `Backbone` | — | Empirical/Declared |
| `compiler.parse::Tok::Cyst` | ctor | `lib/compiler/parse.myc:195` | `Cyst` | — | Empirical/Declared |
| `compiler.parse::Tok::Forage` | ctor | `lib/compiler/parse.myc:195` | `Forage` | — | Empirical/Declared |
| `compiler.parse::Tok::Graft` | ctor | `lib/compiler/parse.myc:195` | `Graft` | — | Empirical/Declared |
| `compiler.parse::Tok::Hypha` | ctor | `lib/compiler/parse.myc:195` | `Hypha` | — | Empirical/Declared |
| `compiler.parse::Tok::Mesh` | ctor | `lib/compiler/parse.myc:195` | `Mesh` | — | Empirical/Declared |
| `compiler.parse::Tok::TFuse` | ctor | `lib/compiler/parse.myc:195` | `TFuse` | — | Empirical/Declared |
| `compiler.parse::Tok::TReclaim` | ctor | `lib/compiler/parse.myc:195` | `TReclaim` | — | Empirical/Declared |
| `compiler.parse::Tok::Tier` | ctor | `lib/compiler/parse.myc:195` | `Tier` | — | Empirical/Declared |
| `compiler.parse::Tok::Xloc` | ctor | `lib/compiler/parse.myc:195` | `Xloc` | — | Empirical/Declared |
| `compiler.parse::Tok::Grow` | ctor | `lib/compiler/parse.myc:196` | `Grow` | — | Empirical/Declared |
| `compiler.parse::Tok::TConsume` | ctor | `lib/compiler/parse.myc:196` | `TConsume` | — | Empirical/Declared |
| `compiler.parse::Tok::TDerive` | ctor | `lib/compiler/parse.myc:196` | `TDerive` | — | Empirical/Declared |
| `compiler.parse::Tok::Matured` | ctor | `lib/compiler/parse.myc:197` | `Matured` | — | Empirical/Declared |
| `compiler.parse::Tok::TFn` | ctor | `lib/compiler/parse.myc:197` | `TFn` | — | Empirical/Declared |
| `compiler.parse::Tok::TImpl` | ctor | `lib/compiler/parse.myc:197` | `TImpl` | — | Empirical/Declared |
| `compiler.parse::Tok::TPub` | ctor | `lib/compiler/parse.myc:197` | `TPub` | — | Empirical/Declared |
| `compiler.parse::Tok::TTrait` | ctor | `lib/compiler/parse.myc:197` | `TTrait` | — | Empirical/Declared |
| `compiler.parse::Tok::TType` | ctor | `lib/compiler/parse.myc:197` | `TType` | — | Empirical/Declared |
| `compiler.parse::Tok::TUse` | ctor | `lib/compiler/parse.myc:197` | `TUse` | — | Empirical/Declared |
| `compiler.parse::Tok::Thaw` | ctor | `lib/compiler/parse.myc:197` | `Thaw` | — | Empirical/Declared |
| `compiler.parse::Tok::Else` | ctor | `lib/compiler/parse.myc:198` | `Else` | — | Empirical/Declared |
| `compiler.parse::Tok::In` | ctor | `lib/compiler/parse.myc:198` | `In` | — | Empirical/Declared |
| `compiler.parse::Tok::TFor` | ctor | `lib/compiler/parse.myc:198` | `TFor` | — | Empirical/Declared |
| `compiler.parse::Tok::TIf` | ctor | `lib/compiler/parse.myc:198` | `TIf` | — | Empirical/Declared |
| `compiler.parse::Tok::TLet` | ctor | `lib/compiler/parse.myc:198` | `TLet` | — | Empirical/Declared |
| `compiler.parse::Tok::TMatch` | ctor | `lib/compiler/parse.myc:198` | `TMatch` | — | Empirical/Declared |
| `compiler.parse::Tok::TSwap` | ctor | `lib/compiler/parse.myc:198` | `TSwap` | — | Empirical/Declared |
| `compiler.parse::Tok::Then` | ctor | `lib/compiler/parse.myc:198` | `Then` | — | Empirical/Declared |
| `compiler.parse::Tok::Paradigm` | ctor | `lib/compiler/parse.myc:199` | `Paradigm` | — | Empirical/Declared |
| `compiler.parse::Tok::Policy` | ctor | `lib/compiler/parse.myc:199` | `Policy` | — | Empirical/Declared |
| `compiler.parse::Tok::TDefault` | ctor | `lib/compiler/parse.myc:199` | `TDefault` | — | Empirical/Declared |
| `compiler.parse::Tok::TSpore` | ctor | `lib/compiler/parse.myc:199` | `TSpore` | — | Empirical/Declared |
| `compiler.parse::Tok::TWild` | ctor | `lib/compiler/parse.myc:199` | `TWild` | — | Empirical/Declared |
| `compiler.parse::Tok::To` | ctor | `lib/compiler/parse.myc:199` | `To` | — | Empirical/Declared |
| `compiler.parse::Tok::With` | ctor | `lib/compiler/parse.myc:199` | `With` | — | Empirical/Declared |
| `compiler.parse::Tok::TLambda` | ctor | `lib/compiler/parse.myc:200` | `TLambda` | — | Empirical/Declared |
| `compiler.parse::Tok::TLower` | ctor | `lib/compiler/parse.myc:200` | `TLower` | — | Empirical/Declared |
| `compiler.parse::Tok::TObject` | ctor | `lib/compiler/parse.myc:200` | `TObject` | — | Empirical/Declared |
| `compiler.parse::Tok::Via` | ctor | `lib/compiler/parse.myc:200` | `Via` | — | Empirical/Declared |
| `compiler.parse::Tok::TKwBinary` | ctor | `lib/compiler/parse.myc:201` | `TKwBinary` | — | Empirical/Declared |
| `compiler.parse::Tok::TKwDense` | ctor | `lib/compiler/parse.myc:201` | `TKwDense` | — | Empirical/Declared |
| `compiler.parse::Tok::TKwTernary` | ctor | `lib/compiler/parse.myc:201` | `TKwTernary` | — | Empirical/Declared |
| `compiler.parse::Tok::TVsa` | ctor | `lib/compiler/parse.myc:201` | `TVsa` | — | Empirical/Declared |
| `compiler.parse::Tok::BinShort` | ctor | `lib/compiler/parse.myc:202` | `BinShort` | — | Empirical/Declared |
| `compiler.parse::Tok::EmbShort` | ctor | `lib/compiler/parse.myc:202` | `EmbShort` | — | Empirical/Declared |
| `compiler.parse::Tok::HvecShort` | ctor | `lib/compiler/parse.myc:202` | `HvecShort` | — | Empirical/Declared |
| `compiler.parse::Tok::TernShort` | ctor | `lib/compiler/parse.myc:202` | `TernShort` | — | Empirical/Declared |
| `compiler.parse::Tok::KwSparse` | ctor | `lib/compiler/parse.myc:203` | `KwSparse` | — | Empirical/Declared |
| `compiler.parse::Tok::TKwBytes` | ctor | `lib/compiler/parse.myc:203` | `TKwBytes` | — | Empirical/Declared |
| `compiler.parse::Tok::TKwFloat` | ctor | `lib/compiler/parse.myc:203` | `TKwFloat` | — | Empirical/Declared |
| `compiler.parse::Tok::TKwSeq` | ctor | `lib/compiler/parse.myc:203` | `TKwSeq` | — | Empirical/Declared |
| `compiler.parse::Tok::TKwSubstrate` | ctor | `lib/compiler/parse.myc:203` | `TKwSubstrate` | — | Empirical/Declared |
| `compiler.parse::Tok::Scalar` | ctor | `lib/compiler/parse.myc:204` | `Scalar(ScalarTok)` | — | Empirical/Declared |
| `compiler.parse::Tok::Strength` | ctor | `lib/compiler/parse.myc:204` | `Strength(StrengthTok)` | — | Empirical/Declared |
| `compiler.parse::Tok::Ident` | ctor | `lib/compiler/parse.myc:205` | `Ident(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Tok::BinLit` | ctor | `lib/compiler/parse.myc:206` | `BinLit(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Tok::BytesLit` | ctor | `lib/compiler/parse.myc:207` | `BytesLit(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Tok::TritLit` | ctor | `lib/compiler/parse.myc:208` | `TritLit(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Tok::StrLit` | ctor | `lib/compiler/parse.myc:209` | `StrLit(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Tok::TInt` | ctor | `lib/compiler/parse.myc:210` | `TInt(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Tok::FloatLit` | ctor | `lib/compiler/parse.myc:211` | `FloatLit(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Tok::LBrace` | ctor | `lib/compiler/parse.myc:212` | `LBrace` | — | Empirical/Declared |
| `compiler.parse::Tok::LBracket` | ctor | `lib/compiler/parse.myc:212` | `LBracket` | — | Empirical/Declared |
| `compiler.parse::Tok::LParen` | ctor | `lib/compiler/parse.myc:212` | `LParen` | — | Empirical/Declared |
| `compiler.parse::Tok::RBrace` | ctor | `lib/compiler/parse.myc:212` | `RBrace` | — | Empirical/Declared |
| `compiler.parse::Tok::RBracket` | ctor | `lib/compiler/parse.myc:212` | `RBracket` | — | Empirical/Declared |
| `compiler.parse::Tok::RParen` | ctor | `lib/compiler/parse.myc:212` | `RParen` | — | Empirical/Declared |
| `compiler.parse::Tok::LAngle` | ctor | `lib/compiler/parse.myc:213` | `LAngle` | — | Empirical/Declared |
| `compiler.parse::Tok::RAngle` | ctor | `lib/compiler/parse.myc:213` | `RAngle` | — | Empirical/Declared |
| `compiler.parse::Tok::Shl` | ctor | `lib/compiler/parse.myc:213` | `Shl` | — | Empirical/Declared |
| `compiler.parse::Tok::Shr` | ctor | `lib/compiler/parse.myc:213` | `Shr` | — | Empirical/Declared |
| `compiler.parse::Tok::At` | ctor | `lib/compiler/parse.myc:214` | `At` | — | Empirical/Declared |
| `compiler.parse::Tok::AtStdSys` | ctor | `lib/compiler/parse.myc:214` | `AtStdSys` | — | Empirical/Declared |
| `compiler.parse::Tok::Colon` | ctor | `lib/compiler/parse.myc:215` | `Colon` | — | Empirical/Declared |
| `compiler.parse::Tok::Comma` | ctor | `lib/compiler/parse.myc:215` | `Comma` | — | Empirical/Declared |
| `compiler.parse::Tok::Dot` | ctor | `lib/compiler/parse.myc:215` | `Dot` | — | Empirical/Declared |
| `compiler.parse::Tok::Pipe` | ctor | `lib/compiler/parse.myc:215` | `Pipe` | — | Empirical/Declared |
| `compiler.parse::Tok::Semi` | ctor | `lib/compiler/parse.myc:215` | `Semi` | — | Empirical/Declared |
| `compiler.parse::Tok::Amp` | ctor | `lib/compiler/parse.myc:216` | `Amp` | — | Empirical/Declared |
| `compiler.parse::Tok::AmpAmp` | ctor | `lib/compiler/parse.myc:216` | `AmpAmp` | — | Empirical/Declared |
| `compiler.parse::Tok::Caret` | ctor | `lib/compiler/parse.myc:216` | `Caret` | — | Empirical/Declared |
| `compiler.parse::Tok::Minus` | ctor | `lib/compiler/parse.myc:216` | `Minus` | — | Empirical/Declared |
| `compiler.parse::Tok::Percent` | ctor | `lib/compiler/parse.myc:216` | `Percent` | — | Empirical/Declared |
| `compiler.parse::Tok::Plus` | ctor | `lib/compiler/parse.myc:216` | `Plus` | — | Empirical/Declared |
| `compiler.parse::Tok::Slash` | ctor | `lib/compiler/parse.myc:216` | `Slash` | — | Empirical/Declared |
| `compiler.parse::Tok::Star` | ctor | `lib/compiler/parse.myc:216` | `Star` | — | Empirical/Declared |
| `compiler.parse::Tok::Arrow` | ctor | `lib/compiler/parse.myc:217` | `Arrow` | — | Empirical/Declared |
| `compiler.parse::Tok::Bang` | ctor | `lib/compiler/parse.myc:217` | `Bang` | — | Empirical/Declared |
| `compiler.parse::Tok::BangEq` | ctor | `lib/compiler/parse.myc:217` | `BangEq` | — | Empirical/Declared |
| `compiler.parse::Tok::Eq` | ctor | `lib/compiler/parse.myc:217` | `Eq` | — | Empirical/Declared |
| `compiler.parse::Tok::EqEq` | ctor | `lib/compiler/parse.myc:217` | `EqEq` | — | Empirical/Declared |
| `compiler.parse::Tok::FatArrow` | ctor | `lib/compiler/parse.myc:217` | `FatArrow` | — | Empirical/Declared |
| `compiler.parse::Tok::PipePipe` | ctor | `lib/compiler/parse.myc:217` | `PipePipe` | — | Empirical/Declared |
| `compiler.parse::Tok::Eof` | ctor | `lib/compiler/parse.myc:218` | `Eof` | — | Empirical/Declared |
| `compiler.parse::Spanned` | type | `lib/compiler/parse.myc:221` | `type Spanned = Sp(Tok, Pos)` | — | Empirical/Declared |
| `compiler.parse::Spanned::Sp` | ctor | `lib/compiler/parse.myc:221` | `Sp(Tok, Pos)` | — | Empirical/Declared |
| `compiler.parse::sp_tok` | fn | `lib/compiler/parse.myc:223` | `fn sp_tok(s: Spanned) => Tok` | — | Empirical/Declared |
| `compiler.parse::sp_pos` | fn | `lib/compiler/parse.myc:226` | `fn sp_pos(s: Spanned) => Pos` | — | Empirical/Declared |
| `compiler.parse::keyword` | fn | `lib/compiler/parse.myc:231` | `fn keyword(word: Bytes) => Option[Tok]` | — | Empirical/Declared |
| `compiler.parse::tok_count` | fn | `lib/compiler/parse.myc:495` | `fn tok_count(v: Vec[Spanned]) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::tok_count_acc` | fn | `lib/compiler/parse.myc:498` | `fn tok_count_acc(v: Vec[Spanned], acc: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::nth` | fn | `lib/compiler/parse.myc:507` | `fn nth(v: Vec[Spanned], n: Binary{32}) => Option[Spanned]` | nth: the token at 0-based index `n`, or `None` if out of range. Lets a differential test inspect ONE token's `Tok`/`Pos` at a time without ever matching multiple `Tok` sites in the same nodule check pass (FLAG-lex-6's mitigation). | Empirical/Declared |
| `compiler.parse::PErr` | type | `lib/compiler/parse.myc:523` | `type PErr = PE(Pos, Bytes)` | — | Empirical/Declared |
| `compiler.parse::PErr::PE` | ctor | `lib/compiler/parse.myc:523` | `PE(Pos, Bytes)` | — | Empirical/Declared |
| `compiler.parse::St` | type | `lib/compiler/parse.myc:527` | `type St = S(Binary{32}, Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.parse::St::S` | ctor | `lib/compiler/parse.myc:527` | `S(Binary{32}, Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.parse::st_i` | fn | `lib/compiler/parse.myc:529` | `fn st_i(s: St) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::st_line` | fn | `lib/compiler/parse.myc:532` | `fn st_line(s: St) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::st_col` | fn | `lib/compiler/parse.myc:535` | `fn st_col(s: St) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::st_pos` | fn | `lib/compiler/parse.myc:538` | `fn st_pos(s: St) => Pos` | — | Empirical/Declared |
| `compiler.parse::empty_bytes` | fn | `lib/compiler/parse.myc:545` | `fn empty_bytes() => Bytes` | — | Empirical/Declared |
| `compiler.parse::bool_and` | fn | `lib/compiler/parse.myc:550` | `fn bool_and(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `compiler.parse::bool_or` | fn | `lib/compiler/parse.myc:553` | `fn bool_or(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `compiler.parse::eq_b` | fn | `lib/compiler/parse.myc:556` | `fn eq_b(a: Binary{8}, b: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::lt_b` | fn | `lib/compiler/parse.myc:559` | `fn lt_b(a: Binary{8}, b: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::le_b` | fn | `lib/compiler/parse.myc:563` | `fn le_b(a: Binary{8}, b: Binary{8}) => Bool` | le_b: a <= b, i.e. NOT (b < a). | Empirical/Declared |
| `compiler.parse::in_range` | fn | `lib/compiler/parse.myc:566` | `fn in_range(c: Binary{8}, lo: Binary{8}, hi: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_cont_byte` | fn | `lib/compiler/parse.myc:570` | `fn is_cont_byte(c: Binary{8}) => Bool` | UTF-8 continuation byte: 0x80..0xBF (mirrors lib/std/text.myc::is_cont_byte exactly). | Empirical/Declared |
| `compiler.parse::is_digit` | fn | `lib/compiler/parse.myc:573` | `fn is_digit(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_upper` | fn | `lib/compiler/parse.myc:576` | `fn is_upper(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_lower` | fn | `lib/compiler/parse.myc:579` | `fn is_lower(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_alpha` | fn | `lib/compiler/parse.myc:582` | `fn is_alpha(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_ident_start` | fn | `lib/compiler/parse.myc:585` | `fn is_ident_start(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_ident_continue` | fn | `lib/compiler/parse.myc:588` | `fn is_ident_continue(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_hex_digit` | fn | `lib/compiler/parse.myc:591` | `fn is_hex_digit(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_bin_digit` | fn | `lib/compiler/parse.myc:594` | `fn is_bin_digit(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_trit_glyph` | fn | `lib/compiler/parse.myc:597` | `fn is_trit_glyph(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_underscore` | fn | `lib/compiler/parse.myc:600` | `fn is_underscore(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_ascii_ws` | fn | `lib/compiler/parse.myc:603` | `fn is_ascii_ws(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_nl_or_cr` | fn | `lib/compiler/parse.myc:606` | `fn is_nl_or_cr(c: Binary{8}) => Bool` | — | Empirical/Declared |
| `compiler.parse::src_peek` | fn | `lib/compiler/parse.myc:610` | `fn src_peek(src: Bytes, i: Binary{32}) => Option[Binary{8}]` | — | Empirical/Declared |
| `compiler.parse::step_pos` | fn | `lib/compiler/parse.myc:615` | `fn step_pos(c: Binary{8}, s: St) => St` | step_pos: the position AFTER consuming byte `c` (FLAG-lex-1: bumps line on '\n', else bumps column only on a non-continuation byte — "count codepoints, not bytes"). | Empirical/Declared |
| `compiler.parse::bump` | fn | `lib/compiler/parse.myc:625` | `fn bump(src: Bytes, s: St) => St` | bump: consume one byte at the current position (never-silent no-op at EOF — total). | Empirical/Declared |
| `compiler.parse::advance_n` | fn | `lib/compiler/parse.myc:629` | `fn advance_n(src: Bytes, s: St, n: Binary{32}) => St` | advance_n: bump `n` times (used after a run-length scan to jump the cursor past a lexeme). | Empirical/Declared |
| `compiler.parse::skip_to_eol` | fn | `lib/compiler/parse.myc:633` | `fn skip_to_eol(src: Bytes, s: St) => St` | — | Empirical/Declared |
| `compiler.parse::skip_trivia` | fn | `lib/compiler/parse.myc:639` | `fn skip_trivia(src: Bytes, s: St) => St` | — | Empirical/Declared |
| `compiler.parse::run_len_ident` | fn | `lib/compiler/parse.myc:658` | `fn run_len_ident(src: Bytes, i: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::run_len_ident_acc` | fn | `lib/compiler/parse.myc:661` | `fn run_len_ident_acc(src: Bytes, i: Binary{32}, acc: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::run_len_digits` | fn | `lib/compiler/parse.myc:670` | `fn run_len_digits(src: Bytes, i: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::run_len_digits_acc` | fn | `lib/compiler/parse.myc:673` | `fn run_len_digits_acc(src: Bytes, i: Binary{32}, acc: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::scan_bin_run` | fn | `lib/compiler/parse.myc:683` | `fn scan_bin_run(src: Bytes, i: Binary{32}) => Pair[Binary{32}, Binary{32}]` | scan_bin_run: Pair(total run length INCLUDING `_` separators, count of actual `0`/`1` digits). | Empirical/Declared |
| `compiler.parse::scan_bin_run_acc` | fn | `lib/compiler/parse.myc:686` | `fn scan_bin_run_acc(src: Bytes, i: Binary{32}, len: Binary{32}, digits: Binary{32}) => Pair[Binary{32}, Binary{32}]` | — | Empirical/Declared |
| `compiler.parse::scan_hex_run` | fn | `lib/compiler/parse.myc:699` | `fn scan_hex_run(src: Bytes, i: Binary{32}) => Pair[Binary{32}, Binary{32}]` | scan_hex_run: Pair(total run length INCLUDING `_` separators, count of actual hex digits). | Empirical/Declared |
| `compiler.parse::scan_hex_run_acc` | fn | `lib/compiler/parse.myc:702` | `fn scan_hex_run_acc(src: Bytes, i: Binary{32}, len: Binary{32}, digits: Binary{32}) => Pair[Binary{32}, Binary{32}]` | — | Empirical/Declared |
| `compiler.parse::scan_trit_run` | fn | `lib/compiler/parse.myc:715` | `fn scan_trit_run(src: Bytes, i: Binary{32}) => Binary{32}` | scan_trit_run: total count of trit glyphs (`+`/`0`/`-`; no `_` separator in the trit grammar). | Empirical/Declared |
| `compiler.parse::scan_trit_run_acc` | fn | `lib/compiler/parse.myc:718` | `fn scan_trit_run_acc(src: Bytes, i: Binary{32}, acc: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::scan_string_escape` | fn | `lib/compiler/parse.myc:730` | `fn scan_string_escape(src: Bytes, s: St) => Result[Pair[Bytes, St], PErr]` | — | Empirical/Declared |
| `compiler.parse::scan_string` | fn | `lib/compiler/parse.myc:747` | `fn scan_string(src: Bytes, s: St) => Result[Pair[Bytes, St], PErr]` | scan_string: `s` is the position right after the opening `"`. Returns the DECODED content + the position right after the closing `"` (never-silent: unterminated / raw newline / bad escape are explicit `Err`, matching lexer.rs::lex_string's contract — see FLAG-lex-7 for message fidelity). | Empirical/Declared |
| `compiler.parse::scan_string_acc` | fn | `lib/compiler/parse.myc:750` | `fn scan_string_acc(src: Bytes, s: St, acc: Bytes) => Result[Pair[Bytes, St], PErr]` | — | Empirical/Declared |
| `compiler.parse::lex_rangle` | fn | `lib/compiler/parse.myc:769` | `fn lex_rangle(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.parse::lex_langle` | fn | `lib/compiler/parse.myc:776` | `fn lex_langle(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.parse::lex_amp` | fn | `lib/compiler/parse.myc:783` | `fn lex_amp(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.parse::lex_pipe` | fn | `lib/compiler/parse.myc:790` | `fn lex_pipe(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.parse::lex_bang` | fn | `lib/compiler/parse.myc:797` | `fn lex_bang(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.parse::lex_eq` | fn | `lib/compiler/parse.myc:804` | `fn lex_eq(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.parse::lex_dash` | fn | `lib/compiler/parse.myc:812` | `fn lex_dash(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.parse::lex_at` | fn | `lib/compiler/parse.myc:821` | `fn lex_at(src: Bytes, s: St) => Pair[Tok, St]` | `@` vs the atomic `@std-sys` nodule-header marker (M-661; mirrors lexer.rs::lex_at exactly, now expressible directly via the `bytes_eq` prim, M-912). | Empirical/Declared |
| `compiler.parse::lex_ident` | fn | `lib/compiler/parse.myc:839` | `fn lex_ident(src: Bytes, s: St) => Pair[Tok, St]` | — | Empirical/Declared |
| `compiler.parse::lex_int` | fn | `lib/compiler/parse.myc:848` | `fn lex_int(src: Bytes, s: St) => Pair[Tok, St]` | lex_int: a non-negative decimal digit run, verbatim (FLAG-lex-2: no float extension, no eager numeric conversion — a structural deviation from lexer.rs, deliberately scoped, no accept-corpus input exercises the gap). | Empirical/Declared |
| `compiler.parse::lex_binlit` | fn | `lib/compiler/parse.myc:853` | `fn lex_binlit(src: Bytes, s: St) => Result[Pair[Tok, St], PErr]` | — | Empirical/Declared |
| `compiler.parse::lex_hexlit` | fn | `lib/compiler/parse.myc:866` | `fn lex_hexlit(src: Bytes, s: St) => Result[Pair[Tok, St], PErr]` | Even-hex-digit-count check (RFC-0032 D4: a byte is two hex chars): `and(digits, 1)` masks the low bit — `0` iff `digits` is even. `and` requires same-width operands; the bare `1` anchors to `digits`'s width (Binary{32}, RFC-0012 ambient bare-decimal inference). | Empirical/Declared |
| `compiler.parse::lex_tritlit` | fn | `lib/compiler/parse.myc:878` | `fn lex_tritlit(src: Bytes, s: St) => Result[Pair[Tok, St], PErr]` | — | Empirical/Declared |
| `compiler.parse::lex_string` | fn | `lib/compiler/parse.myc:886` | `fn lex_string(src: Bytes, s: St) => Result[Pair[Tok, St], PErr]` | — | Empirical/Declared |
| `compiler.parse::lex_zero_prefixed` | fn | `lib/compiler/parse.myc:893` | `fn lex_zero_prefixed(src: Bytes, s: St) => Result[Pair[Tok, St], PErr]` | `0` followed by `b`/`x`/`t` opens a base-prefixed literal; otherwise a plain decimal int. | Empirical/Declared |
| `compiler.parse::next_token` | fn | `lib/compiler/parse.myc:905` | `fn next_token(src: Bytes, s: St) => Result[Pair[Tok, St], PErr]` | — | Empirical/Declared |
| `compiler.parse::run` | fn | `lib/compiler/parse.myc:941` | `fn run(src: Bytes, s: St) => Result[Vec[Spanned], PErr]` | — | Empirical/Declared |
| `compiler.parse::run_acc` | fn | `lib/compiler/parse.myc:944` | `fn run_acc(src: Bytes, s: St, acc: Vec[Spanned]) => Result[Vec[Spanned], PErr]` | — | Empirical/Declared |
| `compiler.parse::lex` | fn | `lib/compiler/parse.myc:959` | `fn lex(src: Bytes) => Result[Vec[Spanned], PErr]` | lex: tokenize `src` into a `Spanned` stream terminated by `Eof` (mirrors lexer.rs::lex). Comments are discarded (FLAG-lex-5); never-silent on any lexically invalid input (an explicit `Err`, never a panic or a silently-skipped character). | Empirical/Declared |
| `compiler.parse::Vis` | type | `lib/compiler/parse.myc:966` | `type Vis = Private \| Pub` | — | Empirical/Declared |
| `compiler.parse::Vis::Private` | ctor | `lib/compiler/parse.myc:966` | `Private` | — | Empirical/Declared |
| `compiler.parse::Vis::Pub` | ctor | `lib/compiler/parse.myc:966` | `Pub` | — | Empirical/Declared |
| `compiler.parse::vis_is_pub` | fn | `lib/compiler/parse.myc:968` | `fn vis_is_pub(v: Vis) => Bool` | — | Empirical/Declared |
| `compiler.parse::Path` | type | `lib/compiler/parse.myc:972` | `type Path = Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.parse::Path::Pth` | ctor | `lib/compiler/parse.myc:972` | `Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.parse::path_segs` | fn | `lib/compiler/parse.myc:974` | `fn path_segs(p: Path) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::UsePath` | type | `lib/compiler/parse.myc:979` | `type UsePath = UP(Path, Bool)` | — | Empirical/Declared |
| `compiler.parse::UsePath::UP` | ctor | `lib/compiler/parse.myc:979` | `UP(Path, Bool)` | — | Empirical/Declared |
| `compiler.parse::usepath_path` | fn | `lib/compiler/parse.myc:981` | `fn usepath_path(u: UsePath) => Path` | — | Empirical/Declared |
| `compiler.parse::usepath_glob` | fn | `lib/compiler/parse.myc:984` | `fn usepath_glob(u: UsePath) => Bool` | — | Empirical/Declared |
| `compiler.parse::Paradigm` | type | `lib/compiler/parse.myc:988` | `type Paradigm = PBinary \| PTernary \| PDense \| PVsa` | — | Empirical/Declared |
| `compiler.parse::Paradigm::PBinary` | ctor | `lib/compiler/parse.myc:988` | `PBinary` | — | Empirical/Declared |
| `compiler.parse::Paradigm::PDense` | ctor | `lib/compiler/parse.myc:988` | `PDense` | — | Empirical/Declared |
| `compiler.parse::Paradigm::PTernary` | ctor | `lib/compiler/parse.myc:988` | `PTernary` | — | Empirical/Declared |
| `compiler.parse::Paradigm::PVsa` | ctor | `lib/compiler/parse.myc:988` | `PVsa` | — | Empirical/Declared |
| `compiler.parse::paradigm_to_bytes` | fn | `lib/compiler/parse.myc:992` | `fn paradigm_to_bytes(p: Paradigm) => Bytes` | paradigm_to_bytes: mirrors `impl Display for Paradigm` (FLAG-ast-7 note: this one IS ported — a trivial fixed 4-string lookup, unlike WidthRef's). | Empirical/Declared |
| `compiler.parse::Scalar` | type | `lib/compiler/parse.myc:1001` | `type Scalar = SF16 \| SBf16 \| SF32 \| SF64` | — | Empirical/Declared |
| `compiler.parse::Scalar::SBf16` | ctor | `lib/compiler/parse.myc:1001` | `SBf16` | — | Empirical/Declared |
| `compiler.parse::Scalar::SF16` | ctor | `lib/compiler/parse.myc:1001` | `SF16` | — | Empirical/Declared |
| `compiler.parse::Scalar::SF32` | ctor | `lib/compiler/parse.myc:1001` | `SF32` | — | Empirical/Declared |
| `compiler.parse::Scalar::SF64` | ctor | `lib/compiler/parse.myc:1001` | `SF64` | — | Empirical/Declared |
| `compiler.parse::Sparsity` | type | `lib/compiler/parse.myc:1004` | `type Sparsity = SpDense \| SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.parse::Sparsity::SpDense` | ctor | `lib/compiler/parse.myc:1004` | `SpDense` | — | Empirical/Declared |
| `compiler.parse::Sparsity::SpSparse` | ctor | `lib/compiler/parse.myc:1004` | `SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.parse::AmbientParams` | type | `lib/compiler/parse.myc:1008` | `type AmbientParams = APSize(Binary{32}) \| APDense(Binary{32}, Scalar) \| APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.parse::AmbientParams::APSize` | ctor | `lib/compiler/parse.myc:1009` | `APSize(Binary{32})` | — | Empirical/Declared |
| `compiler.parse::AmbientParams::APDense` | ctor | `lib/compiler/parse.myc:1010` | `APDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.parse::AmbientParams::APVsa` | ctor | `lib/compiler/parse.myc:1011` | `APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.parse::Strength` | type | `lib/compiler/parse.myc:1014` | `type Strength = GExact \| GProven \| GEmpirical \| GDeclared` | — | Empirical/Declared |
| `compiler.parse::Strength::GDeclared` | ctor | `lib/compiler/parse.myc:1014` | `GDeclared` | — | Empirical/Declared |
| `compiler.parse::Strength::GEmpirical` | ctor | `lib/compiler/parse.myc:1014` | `GEmpirical` | — | Empirical/Declared |
| `compiler.parse::Strength::GExact` | ctor | `lib/compiler/parse.myc:1014` | `GExact` | — | Empirical/Declared |
| `compiler.parse::Strength::GProven` | ctor | `lib/compiler/parse.myc:1014` | `GProven` | — | Empirical/Declared |
| `compiler.parse::strength_rank` | fn | `lib/compiler/parse.myc:1017` | `fn strength_rank(s: Strength) => Binary{8}` | strength_rank: mirrors `Strength::rank` (u8 -> Binary{8}; Declared=0 .. Exact=3). | Empirical/Declared |
| `compiler.parse::strength_meet` | fn | `lib/compiler/parse.myc:1026` | `fn strength_meet(a: Strength, b: Strength) => Strength` | strength_meet: mirrors `Strength::meet` — the weaker (less-trusted) of the two grades. | Empirical/Declared |
| `compiler.parse::strength_satisfies` | fn | `lib/compiler/parse.myc:1033` | `fn strength_satisfies(actual: Strength, demand: Strength) => Bool` | strength_satisfies: mirrors `Strength::satisfies` — `self.rank() >= demand.rank()`. | Empirical/Declared |
| `compiler.parse::WidthRef` | type | `lib/compiler/parse.myc:1040` | `type WidthRef = WLit(Binary{32}) \| WName(Bytes)` | — | Empirical/Declared |
| `compiler.parse::WidthRef::WLit` | ctor | `lib/compiler/parse.myc:1040` | `WLit(Binary{32})` | — | Empirical/Declared |
| `compiler.parse::WidthRef::WName` | ctor | `lib/compiler/parse.myc:1040` | `WName(Bytes)` | — | Empirical/Declared |
| `compiler.parse::ParamKind` | type | `lib/compiler/parse.myc:1046` | `type ParamKind = PkType \| PkWidth` | — | Empirical/Declared |
| `compiler.parse::ParamKind::PkType` | ctor | `lib/compiler/parse.myc:1046` | `PkType` | — | Empirical/Declared |
| `compiler.parse::ParamKind::PkWidth` | ctor | `lib/compiler/parse.myc:1046` | `PkWidth` | — | Empirical/Declared |
| `compiler.parse::paramkind_eq` | fn | `lib/compiler/parse.myc:1048` | `fn paramkind_eq(a: ParamKind, b: ParamKind) => Bool` | — | Empirical/Declared |
| `compiler.parse::TraitRef` | type | `lib/compiler/parse.myc:1055` | `type TraitRef = TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.parse::TraitRef::TRf` | ctor | `lib/compiler/parse.myc:1055` | `TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.parse::traitref_name` | fn | `lib/compiler/parse.myc:1057` | `fn traitref_name(t: TraitRef) => Bytes` | — | Empirical/Declared |
| `compiler.parse::traitref_args` | fn | `lib/compiler/parse.myc:1060` | `fn traitref_args(t: TraitRef) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.parse::TypeParam` | type | `lib/compiler/parse.myc:1064` | `type TypeParam = TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.parse::TypeParam::TP` | ctor | `lib/compiler/parse.myc:1064` | `TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.parse::typeparam_name` | fn | `lib/compiler/parse.myc:1066` | `fn typeparam_name(t: TypeParam) => Bytes` | — | Empirical/Declared |
| `compiler.parse::typeparam_kind` | fn | `lib/compiler/parse.myc:1069` | `fn typeparam_kind(t: TypeParam) => ParamKind` | — | Empirical/Declared |
| `compiler.parse::typeparam_bounds` | fn | `lib/compiler/parse.myc:1072` | `fn typeparam_bounds(t: TypeParam) => Vec[TraitRef]` | — | Empirical/Declared |
| `compiler.parse::typeparam_names_of_kind` | fn | `lib/compiler/parse.myc:1078` | `fn typeparam_names_of_kind(ps: Vec[TypeParam], k: ParamKind) => Vec[Bytes]` | typeparam_names_of_kind: shared filter for fn_sig_param_names / fn_sig_width_param_names below. Non-tail, bounded by the fn's OWN type-parameter count (a handful) — not source length; the same nesting-bounded shape nodule.myc's split_dotted/join_segs use (RFC-0041 §7 W7 amendment 11). | Empirical/Declared |
| `compiler.parse::EffectBudget` | type | `lib/compiler/parse.myc:1089` | `type EffectBudget = EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.parse::EffectBudget::EB` | ctor | `lib/compiler/parse.myc:1089` | `EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.parse::FnSig` | type | `lib/compiler/parse.myc:1094` | `type FnSig = FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.parse::FnSig::FS` | ctor | `lib/compiler/parse.myc:1094` | `FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.parse::fnsig_name` | fn | `lib/compiler/parse.myc:1096` | `fn fnsig_name(s: FnSig) => Bytes` | — | Empirical/Declared |
| `compiler.parse::fnsig_params` | fn | `lib/compiler/parse.myc:1099` | `fn fnsig_params(s: FnSig) => Vec[TypeParam]` | — | Empirical/Declared |
| `compiler.parse::fnsig_value_params` | fn | `lib/compiler/parse.myc:1102` | `fn fnsig_value_params(s: FnSig) => Vec[Param]` | — | Empirical/Declared |
| `compiler.parse::fnsig_ret` | fn | `lib/compiler/parse.myc:1105` | `fn fnsig_ret(s: FnSig) => TypeRef` | — | Empirical/Declared |
| `compiler.parse::fnsig_effects` | fn | `lib/compiler/parse.myc:1108` | `fn fnsig_effects(s: FnSig) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::fnsig_effect_budgets` | fn | `lib/compiler/parse.myc:1111` | `fn fnsig_effect_budgets(s: FnSig) => Vec[EffectBudget]` | — | Empirical/Declared |
| `compiler.parse::fnsig_param_names` | fn | `lib/compiler/parse.myc:1115` | `fn fnsig_param_names(s: FnSig) => Vec[Bytes]` | fnsig_param_names: mirrors `FnSig::param_names` (type-kind params only). | Empirical/Declared |
| `compiler.parse::fnsig_width_param_names` | fn | `lib/compiler/parse.myc:1119` | `fn fnsig_width_param_names(s: FnSig) => Vec[Bytes]` | fnsig_width_param_names: mirrors `FnSig::width_param_names` (width-kind params only). | Empirical/Declared |
| `compiler.parse::Param` | type | `lib/compiler/parse.myc:1123` | `type Param = Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.parse::Param::Prm` | ctor | `lib/compiler/parse.myc:1123` | `Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.parse::param_name` | fn | `lib/compiler/parse.myc:1125` | `fn param_name(p: Param) => Bytes` | — | Empirical/Declared |
| `compiler.parse::param_ty` | fn | `lib/compiler/parse.myc:1128` | `fn param_ty(p: Param) => TypeRef` | — | Empirical/Declared |
| `compiler.parse::TypeRef` | type | `lib/compiler/parse.myc:1134` | `type TypeRef = TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.parse::TypeRef::TR` | ctor | `lib/compiler/parse.myc:1134` | `TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.parse::typeref_base` | fn | `lib/compiler/parse.myc:1136` | `fn typeref_base(t: TypeRef) => BaseType` | — | Empirical/Declared |
| `compiler.parse::typeref_guarantee` | fn | `lib/compiler/parse.myc:1139` | `fn typeref_guarantee(t: TypeRef) => Option[Strength]` | — | Empirical/Declared |
| `compiler.parse::typeref_unguaranteed` | fn | `lib/compiler/parse.myc:1143` | `fn typeref_unguaranteed(b: BaseType) => TypeRef` | typeref_unguaranteed: mirrors `TypeRef::unguaranteed`. | Empirical/Declared |
| `compiler.parse::typeref_with_guarantee` | fn | `lib/compiler/parse.myc:1147` | `fn typeref_with_guarantee(b: BaseType, g: Strength) => TypeRef` | typeref_with_guarantee: mirrors `TypeRef::with_guarantee`. | Empirical/Declared |
| `compiler.parse::BaseType` | type | `lib/compiler/parse.myc:1154` | `type BaseType = KwBinary(WidthRef) \| KwTernary(WidthRef) \| KwDense(Binary{32}, Scalar) \| Vsa(Bytes, Binary{32}, Sparsity) \| KwSubstrate(Bytes) \| KwSeq(TypeRef, Binary{32}) \| KwBytes \| KwFloat \| Named(Bytes, Vec[TypeRef]) \| Ambient(AmbientParams) \| FnArrow(TypeRef, TypeRef) \| Tuple(Vec[TypeRef])` | BaseType: mirrors ast.rs::BaseType field-for-field. FLAG-ast-4 renames the 7 repr-keyword collisions (Kw-prefix); FLAG-ast-5 renames `Fn` (cross-type collision with `Item::Fn`) to `FnArrow`. `Vsa`/`Named`/`Ambient`/`Tuple` are bare (no collision after FLAG-ast-4's Paradigm/ AmbientParams renames leave them the sole survivor). | Empirical/Declared |
| `compiler.parse::BaseType::KwBinary` | ctor | `lib/compiler/parse.myc:1155` | `KwBinary(WidthRef)` | — | Empirical/Declared |
| `compiler.parse::BaseType::KwTernary` | ctor | `lib/compiler/parse.myc:1156` | `KwTernary(WidthRef)` | — | Empirical/Declared |
| `compiler.parse::BaseType::KwDense` | ctor | `lib/compiler/parse.myc:1157` | `KwDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.parse::BaseType::Vsa` | ctor | `lib/compiler/parse.myc:1158` | `Vsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.parse::BaseType::KwSubstrate` | ctor | `lib/compiler/parse.myc:1159` | `KwSubstrate(Bytes)` | — | Empirical/Declared |
| `compiler.parse::BaseType::KwSeq` | ctor | `lib/compiler/parse.myc:1160` | `KwSeq(TypeRef, Binary{32})` | — | Empirical/Declared |
| `compiler.parse::BaseType::KwBytes` | ctor | `lib/compiler/parse.myc:1161` | `KwBytes` | — | Empirical/Declared |
| `compiler.parse::BaseType::KwFloat` | ctor | `lib/compiler/parse.myc:1162` | `KwFloat` | — | Empirical/Declared |
| `compiler.parse::BaseType::Named` | ctor | `lib/compiler/parse.myc:1163` | `Named(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.parse::BaseType::Ambient` | ctor | `lib/compiler/parse.myc:1164` | `Ambient(AmbientParams)` | — | Empirical/Declared |
| `compiler.parse::BaseType::FnArrow` | ctor | `lib/compiler/parse.myc:1165` | `FnArrow(TypeRef, TypeRef)` | — | Empirical/Declared |
| `compiler.parse::BaseType::Tuple` | ctor | `lib/compiler/parse.myc:1166` | `Tuple(Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.parse::ExecutionMode` | type | `lib/compiler/parse.myc:1169` | `type ExecutionMode = Interpreted \| Compiled` | — | Empirical/Declared |
| `compiler.parse::ExecutionMode::Compiled` | ctor | `lib/compiler/parse.myc:1169` | `Compiled` | — | Empirical/Declared |
| `compiler.parse::ExecutionMode::Interpreted` | ctor | `lib/compiler/parse.myc:1169` | `Interpreted` | — | Empirical/Declared |
| `compiler.parse::FnDecl` | type | `lib/compiler/parse.myc:1173` | `type FnDecl = FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.parse::FnDecl::FD` | ctor | `lib/compiler/parse.myc:1173` | `FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.parse::fndecl_vis` | fn | `lib/compiler/parse.myc:1175` | `fn fndecl_vis(f: FnDecl) => Vis` | — | Empirical/Declared |
| `compiler.parse::fndecl_thaw` | fn | `lib/compiler/parse.myc:1178` | `fn fndecl_thaw(f: FnDecl) => Bool` | — | Empirical/Declared |
| `compiler.parse::fndecl_tier` | fn | `lib/compiler/parse.myc:1181` | `fn fndecl_tier(f: FnDecl) => Option[ExecutionMode]` | — | Empirical/Declared |
| `compiler.parse::fndecl_sig` | fn | `lib/compiler/parse.myc:1184` | `fn fndecl_sig(f: FnDecl) => FnSig` | — | Empirical/Declared |
| `compiler.parse::fndecl_body` | fn | `lib/compiler/parse.myc:1187` | `fn fndecl_body(f: FnDecl) => Expr` | — | Empirical/Declared |
| `compiler.parse::Ctor` | type | `lib/compiler/parse.myc:1191` | `type Ctor = Ctr(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.parse::Ctor::Ctr` | ctor | `lib/compiler/parse.myc:1191` | `Ctr(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.parse::ctor_name` | fn | `lib/compiler/parse.myc:1193` | `fn ctor_name(c: Ctor) => Bytes` | — | Empirical/Declared |
| `compiler.parse::ctor_fields` | fn | `lib/compiler/parse.myc:1196` | `fn ctor_fields(c: Ctor) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.parse::TypeDecl` | type | `lib/compiler/parse.myc:1200` | `type TypeDecl = TD(Vis, Bytes, Vec[Bytes], Vec[Ctor])` | — | Empirical/Declared |
| `compiler.parse::TypeDecl::TD` | ctor | `lib/compiler/parse.myc:1200` | `TD(Vis, Bytes, Vec[Bytes], Vec[Ctor])` | — | Empirical/Declared |
| `compiler.parse::typedecl_vis` | fn | `lib/compiler/parse.myc:1202` | `fn typedecl_vis(t: TypeDecl) => Vis` | — | Empirical/Declared |
| `compiler.parse::typedecl_name` | fn | `lib/compiler/parse.myc:1205` | `fn typedecl_name(t: TypeDecl) => Bytes` | — | Empirical/Declared |
| `compiler.parse::typedecl_params` | fn | `lib/compiler/parse.myc:1208` | `fn typedecl_params(t: TypeDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::typedecl_ctors` | fn | `lib/compiler/parse.myc:1211` | `fn typedecl_ctors(t: TypeDecl) => Vec[Ctor]` | — | Empirical/Declared |
| `compiler.parse::TraitDecl` | type | `lib/compiler/parse.myc:1215` | `type TraitDecl = TrD(Vis, Bytes, Vec[Bytes], Vec[FnSig])` | — | Empirical/Declared |
| `compiler.parse::TraitDecl::TrD` | ctor | `lib/compiler/parse.myc:1215` | `TrD(Vis, Bytes, Vec[Bytes], Vec[FnSig])` | — | Empirical/Declared |
| `compiler.parse::traitdecl_vis` | fn | `lib/compiler/parse.myc:1217` | `fn traitdecl_vis(t: TraitDecl) => Vis` | — | Empirical/Declared |
| `compiler.parse::traitdecl_name` | fn | `lib/compiler/parse.myc:1220` | `fn traitdecl_name(t: TraitDecl) => Bytes` | — | Empirical/Declared |
| `compiler.parse::traitdecl_params` | fn | `lib/compiler/parse.myc:1223` | `fn traitdecl_params(t: TraitDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::traitdecl_sigs` | fn | `lib/compiler/parse.myc:1226` | `fn traitdecl_sigs(t: TraitDecl) => Vec[FnSig]` | — | Empirical/Declared |
| `compiler.parse::ImplDecl` | type | `lib/compiler/parse.myc:1230` | `type ImplDecl = ImD(Bytes, Vec[TypeRef], TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.parse::ImplDecl::ImD` | ctor | `lib/compiler/parse.myc:1230` | `ImD(Bytes, Vec[TypeRef], TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.parse::impldecl_trait_name` | fn | `lib/compiler/parse.myc:1232` | `fn impldecl_trait_name(i: ImplDecl) => Bytes` | — | Empirical/Declared |
| `compiler.parse::impldecl_trait_args` | fn | `lib/compiler/parse.myc:1235` | `fn impldecl_trait_args(i: ImplDecl) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.parse::impldecl_for_ty` | fn | `lib/compiler/parse.myc:1238` | `fn impldecl_for_ty(i: ImplDecl) => TypeRef` | — | Empirical/Declared |
| `compiler.parse::impldecl_methods` | fn | `lib/compiler/parse.myc:1241` | `fn impldecl_methods(i: ImplDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.parse::ViaDecl` | type | `lib/compiler/parse.myc:1245` | `type ViaDecl = VD(Binary{32}, Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.parse::ViaDecl::VD` | ctor | `lib/compiler/parse.myc:1245` | `VD(Binary{32}, Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.parse::viadecl_field_idx` | fn | `lib/compiler/parse.myc:1247` | `fn viadecl_field_idx(v: ViaDecl) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::viadecl_trait_name` | fn | `lib/compiler/parse.myc:1250` | `fn viadecl_trait_name(v: ViaDecl) => Bytes` | — | Empirical/Declared |
| `compiler.parse::viadecl_trait_args` | fn | `lib/compiler/parse.myc:1253` | `fn viadecl_trait_args(v: ViaDecl) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.parse::ObjectDecl` | type | `lib/compiler/parse.myc:1257` | `type ObjectDecl = OD(Vis, Bytes, Vec[Bytes], Ctor, Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.parse::ObjectDecl::OD` | ctor | `lib/compiler/parse.myc:1257` | `OD(Vis, Bytes, Vec[Bytes], Ctor, Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.parse::objectdecl_vis` | fn | `lib/compiler/parse.myc:1259` | `fn objectdecl_vis(o: ObjectDecl) => Vis` | — | Empirical/Declared |
| `compiler.parse::objectdecl_name` | fn | `lib/compiler/parse.myc:1262` | `fn objectdecl_name(o: ObjectDecl) => Bytes` | — | Empirical/Declared |
| `compiler.parse::objectdecl_params` | fn | `lib/compiler/parse.myc:1265` | `fn objectdecl_params(o: ObjectDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::objectdecl_ctor` | fn | `lib/compiler/parse.myc:1268` | `fn objectdecl_ctor(o: ObjectDecl) => Ctor` | — | Empirical/Declared |
| `compiler.parse::objectdecl_via_decls` | fn | `lib/compiler/parse.myc:1271` | `fn objectdecl_via_decls(o: ObjectDecl) => Vec[ViaDecl]` | — | Empirical/Declared |
| `compiler.parse::objectdecl_impls` | fn | `lib/compiler/parse.myc:1274` | `fn objectdecl_impls(o: ObjectDecl) => Vec[ImplDecl]` | — | Empirical/Declared |
| `compiler.parse::objectdecl_fns` | fn | `lib/compiler/parse.myc:1277` | `fn objectdecl_fns(o: ObjectDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.parse::InherentImplDecl` | type | `lib/compiler/parse.myc:1281` | `type InherentImplDecl = IID(TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.parse::InherentImplDecl::IID` | ctor | `lib/compiler/parse.myc:1281` | `IID(TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.parse::inherentimpldecl_for_ty` | fn | `lib/compiler/parse.myc:1283` | `fn inherentimpldecl_for_ty(i: InherentImplDecl) => TypeRef` | — | Empirical/Declared |
| `compiler.parse::inherentimpldecl_methods` | fn | `lib/compiler/parse.myc:1286` | `fn inherentimpldecl_methods(i: InherentImplDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.parse::LowerRhs` | type | `lib/compiler/parse.myc:1291` | `type LowerRhs = LRExpr(Expr) \| LRImpl(ImplDecl)` | — | Empirical/Declared |
| `compiler.parse::LowerRhs::LRExpr` | ctor | `lib/compiler/parse.myc:1291` | `LRExpr(Expr)` | — | Empirical/Declared |
| `compiler.parse::LowerRhs::LRImpl` | ctor | `lib/compiler/parse.myc:1291` | `LRImpl(ImplDecl)` | — | Empirical/Declared |
| `compiler.parse::LowerDecl` | type | `lib/compiler/parse.myc:1294` | `type LowerDecl = LD(Bytes, Vec[Bytes], LowerRhs)` | — | Empirical/Declared |
| `compiler.parse::LowerDecl::LD` | ctor | `lib/compiler/parse.myc:1294` | `LD(Bytes, Vec[Bytes], LowerRhs)` | — | Empirical/Declared |
| `compiler.parse::lowerdecl_name` | fn | `lib/compiler/parse.myc:1296` | `fn lowerdecl_name(l: LowerDecl) => Bytes` | — | Empirical/Declared |
| `compiler.parse::lowerdecl_params` | fn | `lib/compiler/parse.myc:1299` | `fn lowerdecl_params(l: LowerDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::lowerdecl_rhs` | fn | `lib/compiler/parse.myc:1302` | `fn lowerdecl_rhs(l: LowerDecl) => LowerRhs` | — | Empirical/Declared |
| `compiler.parse::lowerdecl_expr_rhs` | fn | `lib/compiler/parse.myc:1306` | `fn lowerdecl_expr_rhs(l: LowerDecl) => Option[Expr]` | lowerdecl_expr_rhs: mirrors `LowerDecl::expr_rhs`. | Empirical/Declared |
| `compiler.parse::lowerdecl_impl_rhs` | fn | `lib/compiler/parse.myc:1313` | `fn lowerdecl_impl_rhs(l: LowerDecl) => Option[ImplDecl]` | lowerdecl_impl_rhs: mirrors `LowerDecl::impl_rhs`. | Empirical/Declared |
| `compiler.parse::DeriveDecl` | type | `lib/compiler/parse.myc:1320` | `type DeriveDecl = DD(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.parse::DeriveDecl::DD` | ctor | `lib/compiler/parse.myc:1320` | `DD(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.parse::derivedecl_name` | fn | `lib/compiler/parse.myc:1322` | `fn derivedecl_name(d: DeriveDecl) => Bytes` | — | Empirical/Declared |
| `compiler.parse::derivedecl_for_ty` | fn | `lib/compiler/parse.myc:1325` | `fn derivedecl_for_ty(d: DeriveDecl) => TypeRef` | — | Empirical/Declared |
| `compiler.parse::Item` | type | `lib/compiler/parse.myc:1330` | `type Item = Use(UsePath) \| Default(Paradigm) \| Type(TypeDecl) \| Trait(TraitDecl) \| Impl(ImplDecl) \| Fn(FnDecl) \| Object(ObjectDecl) \| Lower(LowerDecl) \| Derive(DeriveDecl) \| InherentImpl(InherentImplDecl)` | — | Empirical/Declared |
| `compiler.parse::Item::Use` | ctor | `lib/compiler/parse.myc:1331` | `Use(UsePath)` | — | Empirical/Declared |
| `compiler.parse::Item::Default` | ctor | `lib/compiler/parse.myc:1332` | `Default(Paradigm)` | — | Empirical/Declared |
| `compiler.parse::Item::Type` | ctor | `lib/compiler/parse.myc:1333` | `Type(TypeDecl)` | — | Empirical/Declared |
| `compiler.parse::Item::Trait` | ctor | `lib/compiler/parse.myc:1334` | `Trait(TraitDecl)` | — | Empirical/Declared |
| `compiler.parse::Item::Impl` | ctor | `lib/compiler/parse.myc:1335` | `Impl(ImplDecl)` | — | Empirical/Declared |
| `compiler.parse::Item::Fn` | ctor | `lib/compiler/parse.myc:1336` | `Fn(FnDecl)` | — | Empirical/Declared |
| `compiler.parse::Item::Object` | ctor | `lib/compiler/parse.myc:1337` | `Object(ObjectDecl)` | — | Empirical/Declared |
| `compiler.parse::Item::Lower` | ctor | `lib/compiler/parse.myc:1338` | `Lower(LowerDecl)` | — | Empirical/Declared |
| `compiler.parse::Item::Derive` | ctor | `lib/compiler/parse.myc:1339` | `Derive(DeriveDecl)` | — | Empirical/Declared |
| `compiler.parse::Item::InherentImpl` | ctor | `lib/compiler/parse.myc:1340` | `InherentImpl(InherentImplDecl)` | — | Empirical/Declared |
| `compiler.parse::Nodule` | type | `lib/compiler/parse.myc:1344` | `type Nodule = Nd(Path, Bool, Vec[Item])` | — | Empirical/Declared |
| `compiler.parse::Nodule::Nd` | ctor | `lib/compiler/parse.myc:1344` | `Nd(Path, Bool, Vec[Item])` | — | Empirical/Declared |
| `compiler.parse::nodule_path` | fn | `lib/compiler/parse.myc:1346` | `fn nodule_path(n: Nodule) => Path` | — | Empirical/Declared |
| `compiler.parse::nodule_std_sys` | fn | `lib/compiler/parse.myc:1349` | `fn nodule_std_sys(n: Nodule) => Bool` | — | Empirical/Declared |
| `compiler.parse::nodule_items` | fn | `lib/compiler/parse.myc:1352` | `fn nodule_items(n: Nodule) => Vec[Item]` | — | Empirical/Declared |
| `compiler.parse::Phylum` | type | `lib/compiler/parse.myc:1356` | `type Phylum = Phy(Option[Path], Vec[Nodule])` | — | Empirical/Declared |
| `compiler.parse::Phylum::Phy` | ctor | `lib/compiler/parse.myc:1356` | `Phy(Option[Path], Vec[Nodule])` | — | Empirical/Declared |
| `compiler.parse::phylum_path` | fn | `lib/compiler/parse.myc:1358` | `fn phylum_path(p: Phylum) => Option[Path]` | — | Empirical/Declared |
| `compiler.parse::phylum_nodules` | fn | `lib/compiler/parse.myc:1361` | `fn phylum_nodules(p: Phylum) => Vec[Nodule]` | — | Empirical/Declared |
| `compiler.parse::phylum_of_one` | fn | `lib/compiler/parse.myc:1365` | `fn phylum_of_one(n: Nodule) => Phylum` | phylum_of_one: mirrors `Phylum::of_one` — a phylum-of-one wrapping a single bare nodule. | Empirical/Declared |
| `compiler.parse::Literal` | type | `lib/compiler/parse.myc:1371` | `type Literal = Bin(Bytes) \| Trit(Bytes) \| Int(Binary{64}) \| AmbientInt(Paradigm, Binary{64}) \| List(Vec[Expr]) \| LBytes(Bytes) \| Str(Bytes) \| LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Literal::Bin` | ctor | `lib/compiler/parse.myc:1372` | `Bin(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Literal::Trit` | ctor | `lib/compiler/parse.myc:1373` | `Trit(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Literal::Int` | ctor | `lib/compiler/parse.myc:1374` | `Int(Binary{64})` | — | Empirical/Declared |
| `compiler.parse::Literal::AmbientInt` | ctor | `lib/compiler/parse.myc:1375` | `AmbientInt(Paradigm, Binary{64})` | — | Empirical/Declared |
| `compiler.parse::Literal::List` | ctor | `lib/compiler/parse.myc:1376` | `List(Vec[Expr])` | — | Empirical/Declared |
| `compiler.parse::Literal::LBytes` | ctor | `lib/compiler/parse.myc:1377` | `LBytes(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Literal::Str` | ctor | `lib/compiler/parse.myc:1378` | `Str(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Literal::LFloat` | ctor | `lib/compiler/parse.myc:1379` | `LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.parse::literal_binary` | fn | `lib/compiler/parse.myc:1382` | `fn literal_binary(digits: Bytes) => Literal` | literal_binary: mirrors `Literal::binary`. | Empirical/Declared |
| `compiler.parse::literal_ternary` | fn | `lib/compiler/parse.myc:1386` | `fn literal_ternary(trits: Bytes) => Literal` | literal_ternary: mirrors `Literal::ternary`. | Empirical/Declared |
| `compiler.parse::literal_string` | fn | `lib/compiler/parse.myc:1390` | `fn literal_string(content: Bytes) => Literal` | literal_string: mirrors `Literal::string`. | Empirical/Declared |
| `compiler.parse::literal_float` | fn | `lib/compiler/parse.myc:1394` | `fn literal_float(text: Bytes) => Literal` | literal_float: mirrors `Literal::float`. | Empirical/Declared |
| `compiler.parse::Pattern` | type | `lib/compiler/parse.myc:1399` | `type Pattern = PWildcard \| PLit(Literal) \| PCtor(Bytes, Vec[Pattern]) \| PIdent(Bytes) \| PTuple(Vec[Pattern]) \| POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.parse::Pattern::PWildcard` | ctor | `lib/compiler/parse.myc:1400` | `PWildcard` | — | Empirical/Declared |
| `compiler.parse::Pattern::PLit` | ctor | `lib/compiler/parse.myc:1401` | `PLit(Literal)` | — | Empirical/Declared |
| `compiler.parse::Pattern::PCtor` | ctor | `lib/compiler/parse.myc:1402` | `PCtor(Bytes, Vec[Pattern])` | — | Empirical/Declared |
| `compiler.parse::Pattern::PIdent` | ctor | `lib/compiler/parse.myc:1403` | `PIdent(Bytes)` | — | Empirical/Declared |
| `compiler.parse::Pattern::PTuple` | ctor | `lib/compiler/parse.myc:1404` | `PTuple(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.parse::Pattern::POr` | ctor | `lib/compiler/parse.myc:1405` | `POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.parse::Arm` | type | `lib/compiler/parse.myc:1408` | `type Arm = Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.parse::Arm::Ar` | ctor | `lib/compiler/parse.myc:1408` | `Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.parse::arm_pattern` | fn | `lib/compiler/parse.myc:1410` | `fn arm_pattern(a: Arm) => Pattern` | — | Empirical/Declared |
| `compiler.parse::arm_body` | fn | `lib/compiler/parse.myc:1413` | `fn arm_body(a: Arm) => Expr` | — | Empirical/Declared |
| `compiler.parse::Hypha` | type | `lib/compiler/parse.myc:1418` | `type Hypha = Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.parse::Hypha::Hy` | ctor | `lib/compiler/parse.myc:1418` | `Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.parse::hypha_forage` | fn | `lib/compiler/parse.myc:1420` | `fn hypha_forage(h: Hypha) => Option[Expr]` | — | Empirical/Declared |
| `compiler.parse::hypha_body` | fn | `lib/compiler/parse.myc:1423` | `fn hypha_body(h: Hypha) => Expr` | — | Empirical/Declared |
| `compiler.parse::Expr` | type | `lib/compiler/parse.myc:1428` | `type Expr = Let(Bytes, Option[TypeRef], Expr, Expr) \| If(Expr, Expr, Expr) \| Match(Expr, Vec[Arm]) \| For(Bytes, Expr, Bytes, Expr, Expr) \| Swap(Expr, TypeRef, Path) \| WithParadigm(Paradigm, Expr) \| Wild(Expr) \| Spore(Expr) \| Consume(Expr) \| Colony(Vec[Hypha]) \| Lambda(Vec[Param], Expr) \| App(Expr, Vec[Expr]) \| Fuse(Expr, Expr) \| Reclaim(Expr, Expr) \| Path(Path) \| Lit(Literal) \| Ascribe(Expr, TypeRef) \| TupleLit(Vec[Expr])` | — | Empirical/Declared |
| `compiler.parse::Expr::Let` | ctor | `lib/compiler/parse.myc:1429` | `Let(Bytes, Option[TypeRef], Expr, Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::If` | ctor | `lib/compiler/parse.myc:1430` | `If(Expr, Expr, Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::Match` | ctor | `lib/compiler/parse.myc:1431` | `Match(Expr, Vec[Arm])` | — | Empirical/Declared |
| `compiler.parse::Expr::For` | ctor | `lib/compiler/parse.myc:1432` | `For(Bytes, Expr, Bytes, Expr, Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::Path` | ctor | `lib/compiler/parse.myc:1433` | `Path(Path)` | — | Empirical/Declared |
| `compiler.parse::Expr::Swap` | ctor | `lib/compiler/parse.myc:1433` | `Swap(Expr, TypeRef, Path)` | — | Empirical/Declared |
| `compiler.parse::Expr::WithParadigm` | ctor | `lib/compiler/parse.myc:1434` | `WithParadigm(Paradigm, Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::Wild` | ctor | `lib/compiler/parse.myc:1435` | `Wild(Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::Spore` | ctor | `lib/compiler/parse.myc:1436` | `Spore(Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::Consume` | ctor | `lib/compiler/parse.myc:1437` | `Consume(Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::Colony` | ctor | `lib/compiler/parse.myc:1438` | `Colony(Vec[Hypha])` | — | Empirical/Declared |
| `compiler.parse::Expr::Lambda` | ctor | `lib/compiler/parse.myc:1439` | `Lambda(Vec[Param], Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::App` | ctor | `lib/compiler/parse.myc:1440` | `App(Expr, Vec[Expr])` | — | Empirical/Declared |
| `compiler.parse::Expr::Fuse` | ctor | `lib/compiler/parse.myc:1441` | `Fuse(Expr, Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::Reclaim` | ctor | `lib/compiler/parse.myc:1442` | `Reclaim(Expr, Expr)` | — | Empirical/Declared |
| `compiler.parse::Expr::Lit` | ctor | `lib/compiler/parse.myc:1444` | `Lit(Literal)` | — | Empirical/Declared |
| `compiler.parse::Expr::Ascribe` | ctor | `lib/compiler/parse.myc:1445` | `Ascribe(Expr, TypeRef)` | — | Empirical/Declared |
| `compiler.parse::Expr::TupleLit` | ctor | `lib/compiler/parse.myc:1446` | `TupleLit(Vec[Expr])` | — | Empirical/Declared |
| `compiler.parse::is_nodule` | fn | `lib/compiler/parse.myc:1454` | `fn is_nodule(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_phylum` | fn | `lib/compiler/parse.myc:1456` | `fn is_phylum(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_atstdsys` | fn | `lib/compiler/parse.myc:1458` | `fn is_atstdsys(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_semi` | fn | `lib/compiler/parse.myc:1460` | `fn is_semi(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tpub` | fn | `lib/compiler/parse.myc:1462` | `fn is_tpub(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_at` | fn | `lib/compiler/parse.myc:1464` | `fn is_at(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tuse` | fn | `lib/compiler/parse.myc:1466` | `fn is_tuse(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tdefault` | fn | `lib/compiler/parse.myc:1468` | `fn is_tdefault(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_paradigm` | fn | `lib/compiler/parse.myc:1470` | `fn is_paradigm(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_ttype` | fn | `lib/compiler/parse.myc:1472` | `fn is_ttype(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_ttrait` | fn | `lib/compiler/parse.myc:1474` | `fn is_ttrait(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_timpl` | fn | `lib/compiler/parse.myc:1476` | `fn is_timpl(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tfn` | fn | `lib/compiler/parse.myc:1478` | `fn is_tfn(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_thaw` | fn | `lib/compiler/parse.myc:1480` | `fn is_thaw(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_matured` | fn | `lib/compiler/parse.myc:1482` | `fn is_matured(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tcolony` | fn | `lib/compiler/parse.myc:1484` | `fn is_tcolony(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_hypha` | fn | `lib/compiler/parse.myc:1486` | `fn is_hypha(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_mesh` | fn | `lib/compiler/parse.myc:1488` | `fn is_mesh(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_graft` | fn | `lib/compiler/parse.myc:1490` | `fn is_graft(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_cyst` | fn | `lib/compiler/parse.myc:1492` | `fn is_cyst(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_xloc` | fn | `lib/compiler/parse.myc:1494` | `fn is_xloc(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_forage` | fn | `lib/compiler/parse.myc:1496` | `fn is_forage(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_backbone` | fn | `lib/compiler/parse.myc:1498` | `fn is_backbone(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tier` | fn | `lib/compiler/parse.myc:1500` | `fn is_tier(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tfuse` | fn | `lib/compiler/parse.myc:1502` | `fn is_tfuse(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_treclaim` | fn | `lib/compiler/parse.myc:1504` | `fn is_treclaim(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tconsume` | fn | `lib/compiler/parse.myc:1506` | `fn is_tconsume(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_grow` | fn | `lib/compiler/parse.myc:1508` | `fn is_grow(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tlambda` | fn | `lib/compiler/parse.myc:1510` | `fn is_tlambda(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tobject` | fn | `lib/compiler/parse.myc:1512` | `fn is_tobject(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tlower` | fn | `lib/compiler/parse.myc:1514` | `fn is_tlower(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tderive` | fn | `lib/compiler/parse.myc:1516` | `fn is_tderive(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_dot` | fn | `lib/compiler/parse.myc:1518` | `fn is_dot(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_star` | fn | `lib/compiler/parse.myc:1520` | `fn is_star(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_comma` | fn | `lib/compiler/parse.myc:1522` | `fn is_comma(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_lparen` | fn | `lib/compiler/parse.myc:1524` | `fn is_lparen(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_rparen` | fn | `lib/compiler/parse.myc:1526` | `fn is_rparen(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_lbracket` | fn | `lib/compiler/parse.myc:1528` | `fn is_lbracket(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_rbracket` | fn | `lib/compiler/parse.myc:1530` | `fn is_rbracket(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_lbrace` | fn | `lib/compiler/parse.myc:1532` | `fn is_lbrace(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_rbrace` | fn | `lib/compiler/parse.myc:1534` | `fn is_rbrace(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_colon` | fn | `lib/compiler/parse.myc:1536` | `fn is_colon(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_eq` | fn | `lib/compiler/parse.myc:1538` | `fn is_eq(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_fatarrow` | fn | `lib/compiler/parse.myc:1540` | `fn is_fatarrow(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_arrow` | fn | `lib/compiler/parse.myc:1542` | `fn is_arrow(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_bang` | fn | `lib/compiler/parse.myc:1544` | `fn is_bang(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_langle` | fn | `lib/compiler/parse.myc:1546` | `fn is_langle(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_rangle` | fn | `lib/compiler/parse.myc:1548` | `fn is_rangle(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_pipe` | fn | `lib/compiler/parse.myc:1550` | `fn is_pipe(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_plus` | fn | `lib/compiler/parse.myc:1552` | `fn is_plus(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_via` | fn | `lib/compiler/parse.myc:1554` | `fn is_via(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tfor` | fn | `lib/compiler/parse.myc:1556` | `fn is_tfor(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tswap` | fn | `lib/compiler/parse.myc:1558` | `fn is_tswap(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_to` | fn | `lib/compiler/parse.myc:1560` | `fn is_to(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_policy` | fn | `lib/compiler/parse.myc:1562` | `fn is_policy(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_with` | fn | `lib/compiler/parse.myc:1564` | `fn is_with(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_twild` | fn | `lib/compiler/parse.myc:1566` | `fn is_twild(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tspore` | fn | `lib/compiler/parse.myc:1568` | `fn is_tspore(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_in_` | fn | `lib/compiler/parse.myc:1570` | `fn is_in_(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tif` | fn | `lib/compiler/parse.myc:1572` | `fn is_tif(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_then` | fn | `lib/compiler/parse.myc:1574` | `fn is_then(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_else_` | fn | `lib/compiler/parse.myc:1576` | `fn is_else_(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tmatch` | fn | `lib/compiler/parse.myc:1578` | `fn is_tmatch(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tlet` | fn | `lib/compiler/parse.myc:1580` | `fn is_tlet(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_kwsparse` | fn | `lib/compiler/parse.myc:1582` | `fn is_kwsparse(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tkwdense` | fn | `lib/compiler/parse.myc:1584` | `fn is_tkwdense(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_binshort` | fn | `lib/compiler/parse.myc:1586` | `fn is_binshort(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_ternshort` | fn | `lib/compiler/parse.myc:1588` | `fn is_ternshort(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_embshort` | fn | `lib/compiler/parse.myc:1590` | `fn is_embshort(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_hvecshort` | fn | `lib/compiler/parse.myc:1592` | `fn is_hvecshort(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tvsa` | fn | `lib/compiler/parse.myc:1594` | `fn is_tvsa(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tkwsubstrate` | fn | `lib/compiler/parse.myc:1596` | `fn is_tkwsubstrate(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tkwseq` | fn | `lib/compiler/parse.myc:1598` | `fn is_tkwseq(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tkwbytes` | fn | `lib/compiler/parse.myc:1600` | `fn is_tkwbytes(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tkwfloat` | fn | `lib/compiler/parse.myc:1602` | `fn is_tkwfloat(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_shl` | fn | `lib/compiler/parse.myc:1604` | `fn is_shl(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_shr` | fn | `lib/compiler/parse.myc:1606` | `fn is_shr(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_amp` | fn | `lib/compiler/parse.myc:1608` | `fn is_amp(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_caret` | fn | `lib/compiler/parse.myc:1610` | `fn is_caret(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_eqeq` | fn | `lib/compiler/parse.myc:1612` | `fn is_eqeq(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_bangeq` | fn | `lib/compiler/parse.myc:1614` | `fn is_bangeq(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_ampamp` | fn | `lib/compiler/parse.myc:1616` | `fn is_ampamp(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_pipepipe` | fn | `lib/compiler/parse.myc:1618` | `fn is_pipepipe(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_percent` | fn | `lib/compiler/parse.myc:1620` | `fn is_percent(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_slash` | fn | `lib/compiler/parse.myc:1622` | `fn is_slash(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_minus` | fn | `lib/compiler/parse.myc:1624` | `fn is_minus(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tkwbinary` | fn | `lib/compiler/parse.myc:1626` | `fn is_tkwbinary(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_tkwternary` | fn | `lib/compiler/parse.myc:1628` | `fn is_tkwternary(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_eof` | fn | `lib/compiler/parse.myc:1630` | `fn is_eof(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::p_cur` | fn | `lib/compiler/parse.myc:1638` | `fn p_cur(ts: Vec[Spanned]) => Tok` | — | Empirical/Declared |
| `compiler.parse::p_pos` | fn | `lib/compiler/parse.myc:1644` | `fn p_pos(ts: Vec[Spanned]) => Pos` | — | Empirical/Declared |
| `compiler.parse::p_bump` | fn | `lib/compiler/parse.myc:1652` | `fn p_bump(ts: Vec[Spanned]) => Vec[Spanned]` | p_bump: drop the current token UNLESS it is the last remaining one (mirrors Rust's bump, which never advances the index once at the final token — the stream tail is always a lone `Eof` cell). | Empirical/Declared |
| `compiler.parse::p_eat` | fn | `lib/compiler/parse.myc:1663` | `fn p_eat(ts: Vec[Spanned], is_it: Bool) => Option[Vec[Spanned]]` | p_eat(ts, is_it): mirrors `Parser::eat` — the caller supplies `is_it = is_X(p_cur(ts))` (FLAG-parse-5 extended). `Some(ts')` on a match (consumed), `None` otherwise (unchanged). | Empirical/Declared |
| `compiler.parse::p_expect` | fn | `lib/compiler/parse.myc:1671` | `fn p_expect(ts: Vec[Spanned], is_it: Bool, msg: Bytes) => Result[Vec[Spanned], PErr]` | p_expect(ts, is_it, msg): mirrors `Parser::expect` — consumes on a match, else an explicit `PErr` at the CURRENT position (captured by the caller before dispatch, since `ts` is still un-bumped). | Empirical/Declared |
| `compiler.parse::p_ident` | fn | `lib/compiler/parse.myc:1678` | `fn p_ident(ts: Vec[Spanned]) => Result[Pair[Bytes, Vec[Spanned]], PErr]` | p_ident: mirrors `Parser::ident` — an `Ident(s)` token, or an explicit refusal. | Empirical/Declared |
| `compiler.parse::zero32` | fn | `lib/compiler/parse.myc:1690` | `fn zero32() => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::one32` | fn | `lib/compiler/parse.myc:1691` | `fn one32() => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::three32` | fn | `lib/compiler/parse.myc:1692` | `fn three32() => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::zero64` | fn | `lib/compiler/parse.myc:1693` | `fn zero64() => Binary{64}` | — | Empirical/Declared |
| `compiler.parse::one64` | fn | `lib/compiler/parse.myc:1694` | `fn one64() => Binary{64}` | — | Empirical/Declared |
| `compiler.parse::three64` | fn | `lib/compiler/parse.myc:1695` | `fn three64() => Binary{64}` | — | Empirical/Declared |
| `compiler.parse::digit_value` | fn | `lib/compiler/parse.myc:1697` | `fn digit_value(b: Binary{8}) => Binary{8}` | — | Empirical/Declared |
| `compiler.parse::bytes_to_u32` | fn | `lib/compiler/parse.myc:1700` | `fn bytes_to_u32(digits: Bytes, i: Binary{32}, acc: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::decimal_to_u32` | fn | `lib/compiler/parse.myc:1709` | `fn decimal_to_u32(digits: Bytes) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::bytes_to_u64` | fn | `lib/compiler/parse.myc:1712` | `fn bytes_to_u64(digits: Bytes, i: Binary{32}, acc: Binary{64}) => Binary{64}` | — | Empirical/Declared |
| `compiler.parse::decimal_to_u64` | fn | `lib/compiler/parse.myc:1721` | `fn decimal_to_u64(digits: Bytes) => Binary{64}` | — | Empirical/Declared |
| `compiler.parse::p_u32_lit` | fn | `lib/compiler/parse.myc:1725` | `fn p_u32_lit(ts: Vec[Spanned]) => Result[Pair[Binary{32}, Vec[Spanned]], PErr]` | p_u32_lit: mirrors `Parser::u32_lit` — a `TInt(digits)` token, decimal-converted (FLAG-parse-3/4). | Empirical/Declared |
| `compiler.parse::max_expr_depth` | fn | `lib/compiler/parse.myc:1732` | `fn max_expr_depth() => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::enter_depth` | fn | `lib/compiler/parse.myc:1737` | `fn enter_depth(ts: Vec[Spanned], depth: Binary{32}) => Result[Binary{32}, PErr]` | enter_depth: charge one level against the budget. `Err` on overflow (never a panic); the caller passes the CHARGED depth into whatever it recurses into, and simply drops it (does not thread it back out) once that recursion returns — the net-zero restatement of `leave_depth` (FLAG-parse-7). | Empirical/Declared |
| `compiler.parse::contains_bytes` | fn | `lib/compiler/parse.myc:1747` | `fn contains_bytes(xs: Vec[Bytes], x: Bytes) => Bool` | — | Empirical/Declared |
| `compiler.parse::add_bytes_if_absent` | fn | `lib/compiler/parse.myc:1753` | `fn add_bytes_if_absent(xs: Vec[Bytes], x: Bytes) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::NameUses` | type | `lib/compiler/parse.myc:1761` | `type NameUses = NU(Vec[Bytes], Vec[Bytes])` | NameUses: mirrors the pair of BTreeSets `collect_name_uses` accumulates into (width_used, type_used) — threaded functionally instead of via `&mut` out-params. | Empirical/Declared |
| `compiler.parse::NameUses::NU` | ctor | `lib/compiler/parse.myc:1761` | `NU(Vec[Bytes], Vec[Bytes])` | — | Empirical/Declared |
| `compiler.parse::nu_width` | fn | `lib/compiler/parse.myc:1763` | `fn nu_width(n: NameUses) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::nu_type` | fn | `lib/compiler/parse.myc:1766` | `fn nu_type(n: NameUses) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::collect_widthref_use` | fn | `lib/compiler/parse.myc:1769` | `fn collect_widthref_use(w: WidthRef, params: Vec[Bytes], acc: NameUses) => NameUses` | — | Empirical/Declared |
| `compiler.parse::collect_typeref_list_uses` | fn | `lib/compiler/parse.myc:1778` | `fn collect_typeref_list_uses(trs: Vec[TypeRef], params: Vec[Bytes], acc: NameUses) => NameUses` | — | Empirical/Declared |
| `compiler.parse::collect_name_uses` | fn | `lib/compiler/parse.myc:1784` | `fn collect_name_uses(tr: TypeRef, params: Vec[Bytes], acc: NameUses) => NameUses` | — | Empirical/Declared |
| `compiler.parse::collect_base_name_uses` | fn | `lib/compiler/parse.myc:1787` | `fn collect_base_name_uses(bt: BaseType, params: Vec[Bytes], acc: NameUses) => NameUses` | — | Empirical/Declared |
| `compiler.parse::typeparam_names` | fn | `lib/compiler/parse.myc:1802` | `fn typeparam_names(ps: Vec[TypeParam]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::typeparam_names_acc` | fn | `lib/compiler/parse.myc:1805` | `fn typeparam_names_acc(ps: Vec[TypeParam], acc: Vec[Bytes]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.parse::collect_param_list_uses` | fn | `lib/compiler/parse.myc:1811` | `fn collect_param_list_uses(vps: Vec[Param], params: Vec[Bytes], acc: NameUses) => NameUses` | — | Empirical/Declared |
| `compiler.parse::is_bounds_empty` | fn | `lib/compiler/parse.myc:1817` | `fn is_bounds_empty(bs: Vec[TraitRef]) => Bool` | — | Empirical/Declared |
| `compiler.parse::classify_one_param` | fn | `lib/compiler/parse.myc:1820` | `fn classify_one_param(p: TypeParam, width_used: Vec[Bytes], type_used: Vec[Bytes]) => Result[TypeParam, PErr]` | — | Empirical/Declared |
| `compiler.parse::classify_params_list` | fn | `lib/compiler/parse.myc:1835` | `fn classify_params_list(ps: Vec[TypeParam], width_used: Vec[Bytes], type_used: Vec[Bytes]) => Result[Vec[TypeParam], PErr]` | — | Empirical/Declared |
| `compiler.parse::classify_params_list_acc` | fn | `lib/compiler/parse.myc:1838` | `fn classify_params_list_acc(ps: Vec[TypeParam], width_used: Vec[Bytes], type_used: Vec[Bytes], acc: Vec[TypeParam]) => Result[Vec[TypeParam], PErr]` | — | Empirical/Declared |
| `compiler.parse::classify_params` | fn | `lib/compiler/parse.myc:1850` | `fn classify_params(raw_params: Vec[TypeParam], value_params: Vec[Param], ret: TypeRef) => Result[Vec[TypeParam], PErr]` | classify_params: mirrors the free fn of the same name (parse.rs L133). Rust hardcodes `Pos{line:0,col:0}` for these errors (not `self.pos()`) — reproduced verbatim (FLAG-parse-8: error POSITION fidelity is not compared by the Stage-3 gate, only Ok/Err classification). | Empirical/Declared |
| `compiler.parse::first_duplicate_bytes` | fn | `lib/compiler/parse.myc:1857` | `fn first_duplicate_bytes(xs: Vec[Bytes]) => Option[Bytes]` | first_duplicate_bytes: mirrors `first_duplicate_str` (the effect-set / param-name dup check). | Empirical/Declared |
| `compiler.parse::first_duplicate_bytes_seen` | fn | `lib/compiler/parse.myc:1860` | `fn first_duplicate_bytes_seen(xs: Vec[Bytes], seen: Vec[Bytes]) => Option[Bytes]` | — | Empirical/Declared |
| `compiler.parse::parse_paradigm` | fn | `lib/compiler/parse.myc:1872` | `fn parse_paradigm(ts: Vec[Spanned]) => Result[Pair[Paradigm, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_scalar` | fn | `lib/compiler/parse.myc:1881` | `fn parse_scalar(ts: Vec[Spanned]) => Result[Pair[Scalar, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_strength` | fn | `lib/compiler/parse.myc:1892` | `fn parse_strength(ts: Vec[Spanned]) => Result[Pair[Strength, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::braced_u32` | fn | `lib/compiler/parse.myc:1903` | `fn braced_u32(ts: Vec[Spanned]) => Result[Pair[Binary{32}, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_sparsity` | fn | `lib/compiler/parse.myc:1917` | `fn parse_sparsity(ts: Vec[Spanned]) => Result[Pair[Sparsity, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::braced_width` | fn | `lib/compiler/parse.myc:1929` | `fn braced_width(ts: Vec[Spanned]) => Result[Pair[WidthRef, Vec[Spanned]], PErr]` | braced_width: `{` (u32_lit \| Ident) `}` — DN-42/M-753 width slot (concrete literal or width-param name). | Empirical/Declared |
| `compiler.parse::parse_type_ref` | fn | `lib/compiler/parse.myc:1953` | `fn parse_type_ref(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[TypeRef, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_ref_guarded` | fn | `lib/compiler/parse.myc:1959` | `fn parse_type_ref_guarded(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[TypeRef, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_ref_atom` | fn | `lib/compiler/parse.myc:1976` | `fn parse_type_ref_atom(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[TypeRef, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_base_type` | fn | `lib/compiler/parse.myc:1990` | `fn parse_base_type(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[BaseType, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_dense_tail` | fn | `lib/compiler/parse.myc:2056` | `fn parse_dense_tail(ts: Vec[Spanned]) => Result[Pair[BaseType, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_vsa_tail` | fn | `lib/compiler/parse.myc:2078` | `fn parse_vsa_tail(ts: Vec[Spanned]) => Result[Pair[BaseType, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_paren_type_tail` | fn | `lib/compiler/parse.myc:2110` | `fn parse_paren_type_tail(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[BaseType, Vec[Spanned]], PErr]` | parse_paren_type_tail: `ts` is positioned right AFTER the opening `(` (M-826 tuple-type / grouping disambiguation — arity >= 2 is a tuple, a single `(T)` is grouping). | Empirical/Declared |
| `compiler.parse::parse_type_ref_tuple_rest` | fn | `lib/compiler/parse.myc:2132` | `fn parse_type_ref_tuple_rest(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[TypeRef], Vec[Spanned]], PErr]` | parse_type_ref_tuple_rest: the elements after the first comma, tolerating a trailing comma before `)` (mirrors the Rust `while !at(RParen) { push; if !eat(Comma) break }` loop). | Empirical/Declared |
| `compiler.parse::parse_type_ref_tuple_rest_acc` | fn | `lib/compiler/parse.myc:2135` | `fn parse_type_ref_tuple_rest_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[TypeRef]) => Result[Pair[Vec[TypeRef], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_ambient_repr` | fn | `lib/compiler/parse.myc:2156` | `fn parse_ambient_repr(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[AmbientParams, Vec[Spanned]], PErr]` | parse_ambient_repr: `ts` still positioned AT the opening `{` (RFC-0012 SS4.2 paradigm-less repr). | Empirical/Declared |
| `compiler.parse::parse_type_args_opt` | fn | `lib/compiler/parse.myc:2209` | `fn parse_type_args_opt(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[TypeRef], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_ref_comma_list` | fn | `lib/compiler/parse.myc:2225` | `fn parse_type_ref_comma_list(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[TypeRef], Vec[Spanned]], PErr]` | parse_type_ref_comma_list: NON-EMPTY "`one (, one)\*`" — no trailing tolerance (ctor fields, type args; FLAG-parse-5). | Empirical/Declared |
| `compiler.parse::parse_type_ref_comma_list_acc` | fn | `lib/compiler/parse.myc:2228` | `fn parse_type_ref_comma_list_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[TypeRef]) => Result[Pair[Vec[TypeRef], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_literal` | fn | `lib/compiler/parse.myc:2241` | `fn parse_literal(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Literal, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_pattern` | fn | `lib/compiler/parse.myc:2261` | `fn parse_pattern(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Pattern, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_pattern_guarded` | fn | `lib/compiler/parse.myc:2267` | `fn parse_pattern_guarded(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Pattern, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_pattern_lit` | fn | `lib/compiler/parse.myc:2297` | `fn parse_pattern_lit(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Pattern, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_pattern_comma_list` | fn | `lib/compiler/parse.myc:2304` | `fn parse_pattern_comma_list(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Pattern], Vec[Spanned]], PErr]` | parse_pattern_comma_list: NON-EMPTY, no trailing tolerance (ctor sub-patterns). | Empirical/Declared |
| `compiler.parse::parse_pattern_comma_list_acc` | fn | `lib/compiler/parse.myc:2307` | `fn parse_pattern_comma_list_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[Pattern]) => Result[Pair[Vec[Pattern], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_pattern_paren_tail` | fn | `lib/compiler/parse.myc:2319` | `fn parse_pattern_paren_tail(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Pattern, Vec[Spanned]], PErr]` | parse_pattern_paren_tail: `ts` positioned right after `(` (M-826 tuple/grouping disambiguation). | Empirical/Declared |
| `compiler.parse::parse_pattern_tuple_rest` | fn | `lib/compiler/parse.myc:2339` | `fn parse_pattern_tuple_rest(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Pattern], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_pattern_tuple_rest_acc` | fn | `lib/compiler/parse.myc:2342` | `fn parse_pattern_tuple_rest_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[Pattern]) => Result[Pair[Vec[Pattern], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_path` | fn | `lib/compiler/parse.myc:2363` | `fn parse_path(ts: Vec[Spanned]) => Result[Pair[Path, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_path_rest` | fn | `lib/compiler/parse.myc:2372` | `fn parse_path_rest(ts: Vec[Spanned]) => Result[Pair[Vec[Bytes], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_path_rest_acc` | fn | `lib/compiler/parse.myc:2375` | `fn parse_path_rest_acc(ts: Vec[Spanned], acc: Vec[Bytes]) => Result[Pair[Vec[Bytes], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_use_tail` | fn | `lib/compiler/parse.myc:2385` | `fn parse_use_tail(ts: Vec[Spanned]) => Result[Pair[Pair[Vec[Bytes], Bool], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_use_tail_acc` | fn | `lib/compiler/parse.myc:2388` | `fn parse_use_tail_acc(ts: Vec[Spanned], acc: Vec[Bytes]) => Result[Pair[Pair[Vec[Bytes], Bool], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_use` | fn | `lib/compiler/parse.myc:2400` | `fn parse_use(ts: Vec[Spanned]) => Result[Pair[UsePath, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::bytes_eq_b` | fn | `lib/compiler/parse.myc:2413` | `fn bytes_eq_b(a: Bytes, b: Bytes) => Bool` | — | Empirical/Declared |
| `compiler.parse::one8` | fn | `lib/compiler/parse.myc:2416` | `fn one8() => Binary{8}` | — | Empirical/Declared |
| `compiler.parse::is_imperative_word` | fn | `lib/compiler/parse.myc:2421` | `fn is_imperative_word(w: Bytes) => Bool` | — | Empirical/Declared |
| `compiler.parse::is_juxtaposed_opener` | fn | `lib/compiler/parse.myc:2424` | `fn is_juxtaposed_opener(t: Tok) => Bool` | — | Empirical/Declared |
| `compiler.parse::teach_imperative` | fn | `lib/compiler/parse.myc:2431` | `fn teach_imperative(ts: Vec[Spanned]) => Option[PErr]` | — | Empirical/Declared |
| `compiler.parse::infix_op` | fn | `lib/compiler/parse.myc:2444` | `fn infix_op(t: Tok) => Option[Pair[Binary{8}, Bytes]]` | — | Empirical/Declared |
| `compiler.parse::op_call` | fn | `lib/compiler/parse.myc:2465` | `fn op_call(word: Bytes, args: Vec[Expr]) => Expr` | — | Empirical/Declared |
| `compiler.parse::parse_expr` | fn | `lib/compiler/parse.myc:2470` | `fn parse_expr(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_expr_inner` | fn | `lib/compiler/parse.myc:2476` | `fn parse_expr_inner(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_binexpr` | fn | `lib/compiler/parse.myc:2509` | `fn parse_binexpr(ts: Vec[Spanned], depth: Binary{32}, min_bp: Binary{8}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_binexpr_loop` | fn | `lib/compiler/parse.myc:2515` | `fn parse_binexpr_loop(lhs: Expr, ts: Vec[Spanned], depth: Binary{32}, min_bp: Binary{8}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_unary` | fn | `lib/compiler/parse.myc:2531` | `fn parse_unary(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_unary_prefix` | fn | `lib/compiler/parse.myc:2538` | `fn parse_unary_prefix(ts: Vec[Spanned], depth: Binary{32}, word: Bytes) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_for` | fn | `lib/compiler/parse.myc:2549` | `fn parse_for(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_let` | fn | `lib/compiler/parse.myc:2590` | `fn parse_let(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_let_tail` | fn | `lib/compiler/parse.myc:2607` | `fn parse_let_tail(name: Bytes, ty: Option[TypeRef], ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_if` | fn | `lib/compiler/parse.myc:2624` | `fn parse_if(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_match` | fn | `lib/compiler/parse.myc:2649` | `fn parse_match(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_arm_list` | fn | `lib/compiler/parse.myc:2672` | `fn parse_arm_list(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Arm], Vec[Spanned]], PErr]` | parse_arm_list: NON-EMPTY, trailing comma before `}` tolerated. | Empirical/Declared |
| `compiler.parse::parse_arm_list_acc` | fn | `lib/compiler/parse.myc:2675` | `fn parse_arm_list_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[Arm]) => Result[Pair[Vec[Arm], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_arm` | fn | `lib/compiler/parse.myc:2689` | `fn parse_arm(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Arm, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_or_pattern_rest` | fn | `lib/compiler/parse.myc:2703` | `fn parse_or_pattern_rest(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Pattern], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_or_pattern_rest_acc` | fn | `lib/compiler/parse.myc:2706` | `fn parse_or_pattern_rest_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[Pattern]) => Result[Pair[Vec[Pattern], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_arm_tail` | fn | `lib/compiler/parse.myc:2715` | `fn parse_arm_tail(pattern: Pattern, ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Arm, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_swap` | fn | `lib/compiler/parse.myc:2724` | `fn parse_swap(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_with_paradigm` | fn | `lib/compiler/parse.myc:2769` | `fn parse_with_paradigm(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_wild` | fn | `lib/compiler/parse.myc:2794` | `fn parse_wild(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_spore` | fn | `lib/compiler/parse.myc:2811` | `fn parse_spore(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_consume_expr` | fn | `lib/compiler/parse.myc:2828` | `fn parse_consume_expr(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_colony` | fn | `lib/compiler/parse.myc:2840` | `fn parse_colony(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_hypha_list` | fn | `lib/compiler/parse.myc:2858` | `fn parse_hypha_list(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Hypha], Vec[Spanned]], PErr]` | parse_hypha_list: NON-EMPTY, trailing comma before `}` tolerated. | Empirical/Declared |
| `compiler.parse::parse_hypha_list_acc` | fn | `lib/compiler/parse.myc:2861` | `fn parse_hypha_list_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[Hypha]) => Result[Pair[Vec[Hypha], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_hypha` | fn | `lib/compiler/parse.myc:2875` | `fn parse_hypha(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Hypha, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_hypha_tail` | fn | `lib/compiler/parse.myc:2895` | `fn parse_hypha_tail(frg: Option[Expr], ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Hypha, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_app` | fn | `lib/compiler/parse.myc:2904` | `fn parse_app(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_app_tail` | fn | `lib/compiler/parse.myc:2910` | `fn parse_app_tail(e: Expr, ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_expr_list_until` | fn | `lib/compiler/parse.myc:2930` | `fn parse_expr_list_until(ts: Vec[Spanned], depth: Binary{32}, is_end: Bool) => Result[Pair[Vec[Expr], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_expr_comma_list` | fn | `lib/compiler/parse.myc:2936` | `fn parse_expr_comma_list(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Expr], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_expr_comma_list_acc` | fn | `lib/compiler/parse.myc:2939` | `fn parse_expr_comma_list_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[Expr]) => Result[Pair[Vec[Expr], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_primary` | fn | `lib/compiler/parse.myc:2950` | `fn parse_primary(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_primary_lit` | fn | `lib/compiler/parse.myc:2967` | `fn parse_primary_lit(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_primary_paren_tail` | fn | `lib/compiler/parse.myc:2973` | `fn parse_primary_paren_tail(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_expr_tuple_rest` | fn | `lib/compiler/parse.myc:2993` | `fn parse_expr_tuple_rest(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Expr], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_expr_tuple_rest_acc` | fn | `lib/compiler/parse.myc:2996` | `fn parse_expr_tuple_rest_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[Expr]) => Result[Pair[Vec[Expr], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_fuse_expr` | fn | `lib/compiler/parse.myc:3016` | `fn parse_fuse_expr(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_reclaim_expr` | fn | `lib/compiler/parse.myc:3041` | `fn parse_reclaim_expr(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_lambda` | fn | `lib/compiler/parse.myc:3069` | `fn parse_lambda(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Expr, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_params_opt` | fn | `lib/compiler/parse.myc:3101` | `fn parse_params_opt(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Param], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_params_comma_list` | fn | `lib/compiler/parse.myc:3107` | `fn parse_params_comma_list(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Vec[Param], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_params_comma_list_acc` | fn | `lib/compiler/parse.myc:3110` | `fn parse_params_comma_list_acc(ts: Vec[Spanned], depth: Binary{32}, acc: Vec[Param]) => Result[Pair[Vec[Param], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_one_param` | fn | `lib/compiler/parse.myc:3121` | `fn parse_one_param(ts: Vec[Spanned], depth: Binary{32}) => Result[Pair[Param, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_params_opt` | fn | `lib/compiler/parse.myc:3137` | `fn parse_type_params_opt(ts: Vec[Spanned]) => Result[Pair[Vec[Bytes], Vec[Spanned]], PErr]` | parse_type_params_opt: UNBOUNDED names for `type`/`trait` decls; a `: bound` here is an explicit deferred-stage refusal (RFC-0019 SS4.1). | Empirical/Declared |
| `compiler.parse::parse_type_param_name_list` | fn | `lib/compiler/parse.myc:3151` | `fn parse_type_param_name_list(ts: Vec[Spanned]) => Result[Pair[Vec[Bytes], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_param_name_list_acc` | fn | `lib/compiler/parse.myc:3154` | `fn parse_type_param_name_list_acc(ts: Vec[Spanned], acc: Vec[Bytes]) => Result[Pair[Vec[Bytes], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_one_type_param_name` | fn | `lib/compiler/parse.myc:3165` | `fn parse_one_type_param_name(ts: Vec[Spanned]) => Result[Pair[Bytes, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_params_bounded` | fn | `lib/compiler/parse.myc:3177` | `fn parse_type_params_bounded(ts: Vec[Spanned]) => Result[Pair[Vec[TypeParam], Vec[Spanned]], PErr]` | parse_type_params_bounded: BOUNDED type-parameters for function signatures (RFC-0019 SS4.1). | Empirical/Declared |
| `compiler.parse::parse_type_param_comma_list` | fn | `lib/compiler/parse.myc:3191` | `fn parse_type_param_comma_list(ts: Vec[Spanned]) => Result[Pair[Vec[TypeParam], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_param_comma_list_acc` | fn | `lib/compiler/parse.myc:3194` | `fn parse_type_param_comma_list_acc(ts: Vec[Spanned], acc: Vec[TypeParam]) => Result[Pair[Vec[TypeParam], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_param` | fn | `lib/compiler/parse.myc:3205` | `fn parse_type_param(ts: Vec[Spanned]) => Result[Pair[TypeParam, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_bound` | fn | `lib/compiler/parse.myc:3219` | `fn parse_bound(ts: Vec[Spanned]) => Result[Pair[Vec[TraitRef], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_bound_acc` | fn | `lib/compiler/parse.myc:3222` | `fn parse_bound_acc(ts: Vec[Spanned], acc: Vec[TraitRef]) => Result[Pair[Vec[TraitRef], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_trait_ref` | fn | `lib/compiler/parse.myc:3233` | `fn parse_trait_ref(ts: Vec[Spanned]) => Result[Pair[TraitRef, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_const_params_opt` | fn | `lib/compiler/parse.myc:3245` | `fn parse_const_params_opt(ts: Vec[Spanned]) => Result[Pair[Vec[TypeParam], Vec[Spanned]], PErr]` | parse_const_params_opt: `{ Ident (, Ident)\* }?` — width/const parameter DECLARATIONS (distinct `{...}` position from a `Binary{N}` width SLOT — never ambiguous, only ever right after the fn name/[...] and before `(`). | Empirical/Declared |
| `compiler.parse::parse_const_param_comma_list` | fn | `lib/compiler/parse.myc:3259` | `fn parse_const_param_comma_list(ts: Vec[Spanned]) => Result[Pair[Vec[TypeParam], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_const_param_comma_list_acc` | fn | `lib/compiler/parse.myc:3262` | `fn parse_const_param_comma_list_acc(ts: Vec[Spanned], acc: Vec[TypeParam]) => Result[Pair[Vec[TypeParam], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_one_const_param` | fn | `lib/compiler/parse.myc:3273` | `fn parse_one_const_param(ts: Vec[Spanned]) => Result[Pair[TypeParam, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::effect_budget_shift` | fn | `lib/compiler/parse.myc:3285` | `fn effect_budget_shift(unit: Bytes) => Option[Binary{64}]` | — | Empirical/Declared |
| `compiler.parse::append_budget` | fn | `lib/compiler/parse.myc:3297` | `fn append_budget(mb: Option[EffectBudget], rest: Vec[EffectBudget]) => Vec[EffectBudget]` | — | Empirical/Declared |
| `compiler.parse::parse_one_effect_entry` | fn | `lib/compiler/parse.myc:3301` | `fn parse_one_effect_entry(ts: Vec[Spanned]) => Result[Pair[Pair[Bytes, Option[EffectBudget]], Vec[Spanned]], PErr]` | parse_one_effect_entry: `eff (<=N unit?)?` — mirrors the loop body of `parse_effects_opt`. | Empirical/Declared |
| `compiler.parse::parse_effect_budget_tail` | fn | `lib/compiler/parse.myc:3321` | `fn parse_effect_budget_tail(name: Bytes, raw: Binary{64}, budget_pos: Pos, ts: Vec[Spanned]) => Result[Pair[Pair[Bytes, Option[EffectBudget]], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_effect_entries` | fn | `lib/compiler/parse.myc:3343` | `fn parse_effect_entries(ts: Vec[Spanned]) => Result[Pair[Pair[Vec[Bytes], Vec[EffectBudget]], Vec[Spanned]], PErr]` | parse_effect_entries: comma-separated `eff(<=N)?` entries; a trailing comma before `}` is an explicit refusal (kept strict — G2). | Empirical/Declared |
| `compiler.parse::parse_effect_entries_acc` | fn | `lib/compiler/parse.myc:3346` | `fn parse_effect_entries_acc(ts: Vec[Spanned], names: Vec[Bytes], budgets: Vec[EffectBudget]) => Result[Pair[Pair[Vec[Bytes], Vec[EffectBudget]], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_effects_opt` | fn | `lib/compiler/parse.myc:3363` | `fn parse_effects_opt(ts: Vec[Spanned]) => Result[Pair[Pair[Vec[Bytes], Vec[EffectBudget]], Vec[Spanned]], PErr]` | parse_effects_opt: `!{ eff(<=N unit?)? (, ...)\* }?` — absent `!` => pure (empty set). | Empirical/Declared |
| `compiler.parse::Tr3` | type | `lib/compiler/parse.myc:3390` | `type Tr3[A, B, C] = T3(A, B, C)` | — | Empirical/Declared |
| `compiler.parse::Tr3::T3` | ctor | `lib/compiler/parse.myc:3390` | `T3(A, B, C)` | — | Empirical/Declared |
| `compiler.parse::append_typeparams` | fn | `lib/compiler/parse.myc:3394` | `fn append_typeparams(a: Vec[TypeParam], b: Vec[TypeParam]) => Vec[TypeParam]` | append_typeparams: `a ++ b` via double reversal — both passes direct-tail (rev_acc), so the depth is O(1) regardless of the [...]/{...} parameter-list lengths. | Empirical/Declared |
| `compiler.parse::expect_return_arrow` | fn | `lib/compiler/parse.myc:3400` | `fn expect_return_arrow(ts: Vec[Spanned]) => Result[Vec[Spanned], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_sig_tail` | fn | `lib/compiler/parse.myc:3406` | `fn parse_sig_tail(ts: Vec[Spanned]) => Result[Pair[FnSig, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_fn_sig` | fn | `lib/compiler/parse.myc:3455` | `fn parse_fn_sig(ts: Vec[Spanned]) => Result[Pair[FnSig, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_fn_decl` | fn | `lib/compiler/parse.myc:3462` | `fn parse_fn_decl(vis: Vis, ts: Vec[Spanned]) => Result[Pair[FnDecl, Vec[Spanned]], PErr]` | parse_fn_decl: `thaw? fn ...` with the caller-supplied cross-nodule visibility `vis` (M-662). | Empirical/Declared |
| `compiler.parse::parse_fn_decl_tail` | fn | `lib/compiler/parse.myc:3468` | `fn parse_fn_decl_tail(vis: Vis, thw: Bool, ts: Vec[Spanned]) => Result[Pair[FnDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_tier_fn_decl` | fn | `lib/compiler/parse.myc:3486` | `fn parse_tier_fn_decl(vis: Vis, ts: Vec[Spanned]) => Result[Pair[FnDecl, Vec[Spanned]], PErr]` | parse_tier_fn_decl: `@tier(compiled\|interpreted) thaw? fn ...` (DN-58 SSC/M-667). | Empirical/Declared |
| `compiler.parse::parse_tier_fn_decl_tail` | fn | `lib/compiler/parse.myc:3507` | `fn parse_tier_fn_decl_tail(vis: Vis, mode: ExecutionMode, ts: Vec[Spanned]) => Result[Pair[FnDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_tier_fn_decl_tail2` | fn | `lib/compiler/parse.myc:3516` | `fn parse_tier_fn_decl_tail2(vis: Vis, mode: ExecutionMode, thw: Bool, ts: Vec[Spanned]) => Result[Pair[FnDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_ctor` | fn | `lib/compiler/parse.myc:3535` | `fn parse_ctor(ts: Vec[Spanned]) => Result[Pair[Ctor, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_ctor_pipe_list` | fn | `lib/compiler/parse.myc:3554` | `fn parse_ctor_pipe_list(ts: Vec[Spanned]) => Result[Pair[Vec[Ctor], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_ctor_pipe_list_acc` | fn | `lib/compiler/parse.myc:3557` | `fn parse_ctor_pipe_list_acc(ts: Vec[Spanned], acc: Vec[Ctor]) => Result[Pair[Vec[Ctor], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_type_decl` | fn | `lib/compiler/parse.myc:3568` | `fn parse_type_decl(vis: Vis, ts: Vec[Spanned]) => Result[Pair[TypeDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_trait_sig_list` | fn | `lib/compiler/parse.myc:3590` | `fn parse_trait_sig_list(ts: Vec[Spanned]) => Result[Pair[Vec[FnSig], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_trait_sig_list_acc` | fn | `lib/compiler/parse.myc:3593` | `fn parse_trait_sig_list_acc(ts: Vec[Spanned], acc: Vec[FnSig]) => Result[Pair[Vec[FnSig], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_trait_decl` | fn | `lib/compiler/parse.myc:3607` | `fn parse_trait_decl(vis: Vis, ts: Vec[Spanned]) => Result[Pair[TraitDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_impl_method_list` | fn | `lib/compiler/parse.myc:3636` | `fn parse_impl_method_list(ts: Vec[Spanned]) => Result[Pair[Vec[FnDecl], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_impl_method_list_acc` | fn | `lib/compiler/parse.myc:3639` | `fn parse_impl_method_list_acc(ts: Vec[Spanned], acc: Vec[FnDecl]) => Result[Pair[Vec[FnDecl], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_impl_body` | fn | `lib/compiler/parse.myc:3656` | `fn parse_impl_body(ts: Vec[Spanned]) => Result[Pair[Vec[FnDecl], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_impl_decl` | fn | `lib/compiler/parse.myc:3670` | `fn parse_impl_decl(ts: Vec[Spanned]) => Result[Pair[ImplDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_impl_item` | fn | `lib/compiler/parse.myc:3698` | `fn parse_impl_item(ts: Vec[Spanned]) => Result[Pair[Item, Vec[Spanned]], PErr]` | parse_impl_item: the top-level `impl` dispatcher (M-664) — disambiguates the trait-instance form (`impl Trait for T { ... }`) from the inherent-method form (`impl T { ... }`) by parsing the head base type once and branching on the follower (`for` vs `{`). | Empirical/Declared |
| `compiler.parse::parse_impl_item_tail` | fn | `lib/compiler/parse.myc:3707` | `fn parse_impl_item_tail(head: BaseType, ts: Vec[Spanned]) => Result[Pair[Item, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_object_body` | fn | `lib/compiler/parse.myc:3732` | `fn parse_object_body(ts: Vec[Spanned]) => Result[Pair[Tr3[Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl]], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_object_body_acc` | fn | `lib/compiler/parse.myc:3738` | `fn parse_object_body_acc(ts: Vec[Spanned], vds: Vec[ViaDecl], ims: Vec[ImplDecl], fs: Vec[FnDecl]) => Result[Pair[Tr3[Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl]], Vec[Spanned]], PErr]` | parse_object_body_acc: the accumulator dispatcher — each member arm parses ONE member (incl. its `;` terminator) then direct-tail-recurses with the member prepended; the three lists are rev_acc'd back to source order once at the `}`. | Empirical/Declared |
| `compiler.parse::parse_one_object_via` | fn | `lib/compiler/parse.myc:3764` | `fn parse_one_object_via(ts: Vec[Spanned]) => Result[Pair[ViaDecl, Vec[Spanned]], PErr]` | parse_one_object_via: ONE `via N : Trait[args]?;` clause — `ts` positioned right after `via`. | Empirical/Declared |
| `compiler.parse::parse_one_object_impl` | fn | `lib/compiler/parse.myc:3788` | `fn parse_one_object_impl(ts: Vec[Spanned]) => Result[Pair[ImplDecl, Vec[Spanned]], PErr]` | parse_one_object_impl: ONE `impl Trait for T { ... };` member. | Empirical/Declared |
| `compiler.parse::parse_one_object_fn` | fn | `lib/compiler/parse.myc:3800` | `fn parse_one_object_fn(ts: Vec[Spanned]) => Result[Pair[FnDecl, Vec[Spanned]], PErr]` | parse_one_object_fn: ONE `fn ...;` / `thaw fn ...;` member. | Empirical/Declared |
| `compiler.parse::parse_object_decl` | fn | `lib/compiler/parse.myc:3811` | `fn parse_object_decl(vis: Vis, ts: Vec[Spanned]) => Result[Pair[ObjectDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_lower_item_rhs` | fn | `lib/compiler/parse.myc:3851` | `fn parse_lower_item_rhs(ts: Vec[Spanned]) => Result[Pair[ImplDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_lower_decl` | fn | `lib/compiler/parse.myc:3854` | `fn parse_lower_decl(ts: Vec[Spanned]) => Result[Pair[LowerDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_derive_decl` | fn | `lib/compiler/parse.myc:3882` | `fn parse_derive_decl(ts: Vec[Spanned]) => Result[Pair[DeriveDecl, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_item` | fn | `lib/compiler/parse.myc:3901` | `fn parse_item(ts: Vec[Spanned]) => Result[Pair[Item, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_pub_item` | fn | `lib/compiler/parse.myc:3946` | `fn parse_pub_item(ts: Vec[Spanned]) => Result[Pair[Item, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_nodule_items` | fn | `lib/compiler/parse.myc:3968` | `fn parse_nodule_items(ts: Vec[Spanned]) => Result[Pair[Vec[Item], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_nodule_items_acc` | fn | `lib/compiler/parse.myc:3971` | `fn parse_nodule_items_acc(ts: Vec[Spanned], acc: Vec[Item]) => Result[Pair[Vec[Item], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_nodule_tail` | fn | `lib/compiler/parse.myc:3985` | `fn parse_nodule_tail(path: Path, std_sys: Bool, ts: Vec[Spanned]) => Result[Pair[Nodule, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_nodule` | fn | `lib/compiler/parse.myc:3994` | `fn parse_nodule(ts: Vec[Spanned]) => Result[Pair[Nodule, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_nodule_list` | fn | `lib/compiler/parse.myc:4008` | `fn parse_nodule_list(ts: Vec[Spanned]) => Result[Pair[Vec[Nodule], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_nodule_list_acc` | fn | `lib/compiler/parse.myc:4011` | `fn parse_nodule_list_acc(ts: Vec[Spanned], acc: Vec[Nodule]) => Result[Pair[Vec[Nodule], Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_phylum_tail` | fn | `lib/compiler/parse.myc:4020` | `fn parse_phylum_tail(path: Option[Path], ts: Vec[Spanned]) => Result[Pair[Phylum, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_phylum_prog` | fn | `lib/compiler/parse.myc:4029` | `fn parse_phylum_prog(ts: Vec[Spanned]) => Result[Pair[Phylum, Vec[Spanned]], PErr]` | — | Empirical/Declared |
| `compiler.parse::parse` | fn | `lib/compiler/parse.myc:4040` | `fn parse(src: Bytes) => Result[Nodule, PErr]` | — | Empirical/Declared |
| `compiler.parse::parse_phylum` | fn | `lib/compiler/parse.myc:4054` | `fn parse_phylum(src: Bytes) => Result[Phylum, PErr]` | — | Empirical/Declared |
| `compiler.parse::Fp` | type | `lib/compiler/parse.myc:4080` | `type Fp = FP(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.parse::Fp::FP` | ctor | `lib/compiler/parse.myc:4080` | `FP(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.parse::fp_hash` | fn | `lib/compiler/parse.myc:4082` | `fn fp_hash(fp: Fp) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::fp_count` | fn | `lib/compiler/parse.myc:4083` | `fn fp_count(fp: Fp) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::rotl7` | fn | `lib/compiler/parse.myc:4085` | `fn rotl7(x: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::fp_tag` | fn | `lib/compiler/parse.myc:4088` | `fn fp_tag(fp: Fp, tag: Binary{32}) => Fp` | — | Empirical/Declared |
| `compiler.parse::fp_bytes` | fn | `lib/compiler/parse.myc:4091` | `fn fp_bytes(fp: Fp, b: Bytes) => Fp` | — | Empirical/Declared |
| `compiler.parse::fp_u32` | fn | `lib/compiler/parse.myc:4094` | `fn fp_u32(fp: Fp, n: Binary{32}) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_vis` | fn | `lib/compiler/parse.myc:4098` | `fn walk_vis(fp: Fp, v: Vis) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_paradigm` | fn | `lib/compiler/parse.myc:4101` | `fn walk_paradigm(fp: Fp, p: Paradigm) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_scalar` | fn | `lib/compiler/parse.myc:4109` | `fn walk_scalar(fp: Fp, s: Scalar) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_strength` | fn | `lib/compiler/parse.myc:4117` | `fn walk_strength(fp: Fp, s: Strength) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_sparsity` | fn | `lib/compiler/parse.myc:4125` | `fn walk_sparsity(fp: Fp, s: Sparsity) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_paramkind` | fn | `lib/compiler/parse.myc:4131` | `fn walk_paramkind(fp: Fp, k: ParamKind) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_execmode` | fn | `lib/compiler/parse.myc:4134` | `fn walk_execmode(fp: Fp, e: ExecutionMode) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_widthref` | fn | `lib/compiler/parse.myc:4137` | `fn walk_widthref(fp: Fp, w: WidthRef) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_path` | fn | `lib/compiler/parse.myc:4143` | `fn walk_path(fp: Fp, p: Path) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_bytes_list` | fn | `lib/compiler/parse.myc:4146` | `fn walk_bytes_list(fp: Fp, xs: Vec[Bytes]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_usepath` | fn | `lib/compiler/parse.myc:4149` | `fn walk_usepath(fp: Fp, u: UsePath) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_bool` | fn | `lib/compiler/parse.myc:4152` | `fn walk_bool(fp: Fp, b: Bool) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_ambientparams` | fn | `lib/compiler/parse.myc:4156` | `fn walk_ambientparams(fp: Fp, a: AmbientParams) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_typeref` | fn | `lib/compiler/parse.myc:4163` | `fn walk_typeref(fp: Fp, t: TypeRef) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_guarantee_opt` | fn | `lib/compiler/parse.myc:4166` | `fn walk_guarantee_opt(fp: Fp, g: Option[Strength]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_typeref_list` | fn | `lib/compiler/parse.myc:4169` | `fn walk_typeref_list(fp: Fp, xs: Vec[TypeRef]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_basetype` | fn | `lib/compiler/parse.myc:4172` | `fn walk_basetype(fp: Fp, b: BaseType) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_traitref` | fn | `lib/compiler/parse.myc:4188` | `fn walk_traitref(fp: Fp, t: TraitRef) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_traitref_list` | fn | `lib/compiler/parse.myc:4191` | `fn walk_traitref_list(fp: Fp, xs: Vec[TraitRef]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_typeparam` | fn | `lib/compiler/parse.myc:4194` | `fn walk_typeparam(fp: Fp, t: TypeParam) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_typeparam_list` | fn | `lib/compiler/parse.myc:4197` | `fn walk_typeparam_list(fp: Fp, xs: Vec[TypeParam]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_param` | fn | `lib/compiler/parse.myc:4200` | `fn walk_param(fp: Fp, p: Param) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_param_list` | fn | `lib/compiler/parse.myc:4203` | `fn walk_param_list(fp: Fp, xs: Vec[Param]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_bytes_list2` | fn | `lib/compiler/parse.myc:4206` | `fn walk_bytes_list2(fp: Fp, xs: Vec[Bytes]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_effectbudget_list` | fn | `lib/compiler/parse.myc:4213` | `fn walk_effectbudget_list(fp: Fp, xs: Vec[EffectBudget]) => Fp` | FLAG-parse-10: the budget VALUE is not mixed into the fingerprint (only its effect name) -- narrowing a Binary{64} budget to Binary{32} for the hash risks the SAME checked-narrow refusal as FLAG-parse-4/-9, and no accept-corpus file uses `(<=N)` syntax at all (verified during authoring), so this is an honest, unexercised, zero-cost-to-skip narrowing. | Empirical/Declared |
| `compiler.parse::walk_fnsig` | fn | `lib/compiler/parse.myc:4216` | `fn walk_fnsig(fp: Fp, s: FnSig) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_ctor` | fn | `lib/compiler/parse.myc:4229` | `fn walk_ctor(fp: Fp, c: Ctor) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_ctor_list` | fn | `lib/compiler/parse.myc:4232` | `fn walk_ctor_list(fp: Fp, xs: Vec[Ctor]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_typedecl` | fn | `lib/compiler/parse.myc:4235` | `fn walk_typedecl(fp: Fp, t: TypeDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_fnsig_list` | fn | `lib/compiler/parse.myc:4238` | `fn walk_fnsig_list(fp: Fp, xs: Vec[FnSig]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_traitdecl` | fn | `lib/compiler/parse.myc:4241` | `fn walk_traitdecl(fp: Fp, t: TraitDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_fndecl` | fn | `lib/compiler/parse.myc:4244` | `fn walk_fndecl(fp: Fp, f: FnDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_tier_opt` | fn | `lib/compiler/parse.myc:4249` | `fn walk_tier_opt(fp: Fp, t: Option[ExecutionMode]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_fndecl_list` | fn | `lib/compiler/parse.myc:4252` | `fn walk_fndecl_list(fp: Fp, xs: Vec[FnDecl]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_impldecl` | fn | `lib/compiler/parse.myc:4255` | `fn walk_impldecl(fp: Fp, i: ImplDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_viadecl` | fn | `lib/compiler/parse.myc:4260` | `fn walk_viadecl(fp: Fp, v: ViaDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_viadecl_list` | fn | `lib/compiler/parse.myc:4263` | `fn walk_viadecl_list(fp: Fp, xs: Vec[ViaDecl]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_impldecl_list` | fn | `lib/compiler/parse.myc:4266` | `fn walk_impldecl_list(fp: Fp, xs: Vec[ImplDecl]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_objectdecl` | fn | `lib/compiler/parse.myc:4269` | `fn walk_objectdecl(fp: Fp, o: ObjectDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_inherentimpldecl` | fn | `lib/compiler/parse.myc:4280` | `fn walk_inherentimpldecl(fp: Fp, i: InherentImplDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_lowerrhs` | fn | `lib/compiler/parse.myc:4283` | `fn walk_lowerrhs(fp: Fp, r: LowerRhs) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_lowerdecl` | fn | `lib/compiler/parse.myc:4289` | `fn walk_lowerdecl(fp: Fp, l: LowerDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_derivedecl` | fn | `lib/compiler/parse.myc:4292` | `fn walk_derivedecl(fp: Fp, d: DeriveDecl) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_expr_list` | fn | `lib/compiler/parse.myc:4296` | `fn walk_expr_list(fp: Fp, xs: Vec[Expr]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_literal` | fn | `lib/compiler/parse.myc:4299` | `fn walk_literal(fp: Fp, l: Literal) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_pattern` | fn | `lib/compiler/parse.myc:4311` | `fn walk_pattern(fp: Fp, p: Pattern) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_pattern_list` | fn | `lib/compiler/parse.myc:4321` | `fn walk_pattern_list(fp: Fp, xs: Vec[Pattern]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_arm` | fn | `lib/compiler/parse.myc:4324` | `fn walk_arm(fp: Fp, a: Arm) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_arm_list` | fn | `lib/compiler/parse.myc:4327` | `fn walk_arm_list(fp: Fp, xs: Vec[Arm]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_expr_opt` | fn | `lib/compiler/parse.myc:4330` | `fn walk_expr_opt(fp: Fp, e: Option[Expr]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_hypha` | fn | `lib/compiler/parse.myc:4333` | `fn walk_hypha(fp: Fp, h: Hypha) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_hypha_list` | fn | `lib/compiler/parse.myc:4336` | `fn walk_hypha_list(fp: Fp, xs: Vec[Hypha]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_typeref_opt` | fn | `lib/compiler/parse.myc:4339` | `fn walk_typeref_opt(fp: Fp, t: Option[TypeRef]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_expr` | fn | `lib/compiler/parse.myc:4342` | `fn walk_expr(fp: Fp, e: Expr) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_item` | fn | `lib/compiler/parse.myc:4365` | `fn walk_item(fp: Fp, i: Item) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_item_list` | fn | `lib/compiler/parse.myc:4379` | `fn walk_item_list(fp: Fp, xs: Vec[Item]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_nodule` | fn | `lib/compiler/parse.myc:4382` | `fn walk_nodule(fp: Fp, n: Nodule) => Fp` | — | Empirical/Declared |
| `compiler.parse::fingerprint_nodule` | fn | `lib/compiler/parse.myc:4386` | `fn fingerprint_nodule(n: Nodule) => Fp` | fingerprint_nodule: the top-level entry — a fresh (0,0) accumulator walked over a parsed Nodule. | Empirical/Declared |
| `compiler.parse::parse_ok_code` | fn | `lib/compiler/parse.myc:4393` | `fn parse_ok_code(src: Bytes) => Binary{32}` | parse_ok_code: 1 if `src` parses Ok, 0 if Err (the classification-parity leg). | Empirical/Declared |
| `compiler.parse::parse_fingerprint_hash` | fn | `lib/compiler/parse.myc:4398` | `fn parse_fingerprint_hash(src: Bytes) => Binary{32}` | parse_fingerprint_hash / parse_fingerprint_count: the fingerprint's two legs, `Eof`-safe default (0) on a parse error (the caller only compares these on inputs BOTH sides accepted). | Empirical/Declared |
| `compiler.parse::parse_fingerprint_count` | fn | `lib/compiler/parse.myc:4401` | `fn parse_fingerprint_count(src: Bytes) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::stage3_verdict` | fn | `lib/compiler/parse.myc:4410` | `fn stage3_verdict(src: Bytes, want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | stage3_verdict: the ONE-EVAL-PER-FILE Stage-3 gate driver (DN-26 SS7.3 runtime-budget economy: the Rust harness checks THIS nodule once, then calls this fn once per corpus file with the oracle-computed expectations as arguments — one elaboration, many eval calls). Verdict codes: 1 = full agreement; 0 = Ok/Err classification mismatch; 2 = fingerprint HASH mismatch; 3 = fingerprint NODE-COUNT mismatch. For a want_ok=0 (oracle-rejected) input the fingerprint legs are vacuous (never compared — DN-26 SS7.3 compares shape only on mutually-accepted files). | Empirical/Declared |
| `compiler.parse::stage3_verdict_fp` | fn | `lib/compiler/parse.myc:4419` | `fn stage3_verdict_fp(fp: Fp, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `compiler.parse::walk_phylum_path_opt` | fn | `lib/compiler/parse.myc:4428` | `fn walk_phylum_path_opt(fp: Fp, p: Option[Path]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_nodule_list` | fn | `lib/compiler/parse.myc:4431` | `fn walk_nodule_list(fp: Fp, xs: Vec[Nodule]) => Fp` | — | Empirical/Declared |
| `compiler.parse::walk_phylum` | fn | `lib/compiler/parse.myc:4434` | `fn walk_phylum(fp: Fp, ph: Phylum) => Fp` | — | Empirical/Declared |
| `compiler.parse::fingerprint_phylum` | fn | `lib/compiler/parse.myc:4437` | `fn fingerprint_phylum(ph: Phylum) => Fp` | — | Empirical/Declared |
| `compiler.parse::stage3_phylum_verdict` | fn | `lib/compiler/parse.myc:4442` | `fn stage3_phylum_verdict(src: Bytes, want_ok: Binary{32}, want_hash: Binary{32}, want_count: Binary{32}) => Binary{32}` | stage3_phylum_verdict: the parse_phylum leg of the Stage-3 gate (same verdict codes as stage3_verdict; same one-eval-per-file economy). | Empirical/Declared |

### compiler.semcore

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.semcore` | nodule | `lib/compiler/semcore.myc:191` | `nodule compiler.semcore` | Self-hosted L1 semantic-core — a PARTIAL FIRST INCREMENT (M-740 Stage 5, increment 1; DN-26 SS7.3 row 5 / SS9 flag-1). The full semcore is a 9-file strongly-connected component (checkty/elab/eval/mono/fuse/decision/usefulness/grade/affine, ~16.7k Rust lines) that is ONE | Empirical/Declared |
| `compiler.semcore::Option` | type | `lib/compiler/semcore.myc:194` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.semcore::Option::None` | ctor | `lib/compiler/semcore.myc:194` | `None` | — | Empirical/Declared |
| `compiler.semcore::Option::Some` | ctor | `lib/compiler/semcore.myc:194` | `Some(A)` | — | Empirical/Declared |
| `compiler.semcore::Result` | type | `lib/compiler/semcore.myc:195` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `compiler.semcore::Result::Err` | ctor | `lib/compiler/semcore.myc:195` | `Err(E)` | — | Empirical/Declared |
| `compiler.semcore::Result::Ok` | ctor | `lib/compiler/semcore.myc:195` | `Ok(A)` | — | Empirical/Declared |
| `compiler.semcore::Vec` | type | `lib/compiler/semcore.myc:196` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.semcore::Vec::Cons` | ctor | `lib/compiler/semcore.myc:196` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.semcore::Vec::Nil` | ctor | `lib/compiler/semcore.myc:196` | `Nil` | — | Empirical/Declared |
| `compiler.semcore::Pair` | type | `lib/compiler/semcore.myc:197` | `type Pair[A, B] = Pr(A, B)` | — | Empirical/Declared |
| `compiler.semcore::Pair::Pr` | ctor | `lib/compiler/semcore.myc:197` | `Pr(A, B)` | — | Empirical/Declared |
| `compiler.semcore::Vis` | type | `lib/compiler/semcore.myc:202` | `type Vis = Private \| Pub` | Vis: mirrors ast.myc::Vis (a field of `FnDecl`; grade never inspects its value). | Empirical/Declared |
| `compiler.semcore::Vis::Private` | ctor | `lib/compiler/semcore.myc:202` | `Private` | — | Empirical/Declared |
| `compiler.semcore::Vis::Pub` | ctor | `lib/compiler/semcore.myc:202` | `Pub` | — | Empirical/Declared |
| `compiler.semcore::Path` | type | `lib/compiler/semcore.myc:205` | `type Path = Pth(Vec[Bytes])` | Path: mirrors ast.myc::Path (a dotted path or bare name). | Empirical/Declared |
| `compiler.semcore::Path::Pth` | ctor | `lib/compiler/semcore.myc:205` | `Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.semcore::path_segs` | fn | `lib/compiler/semcore.myc:207` | `fn path_segs(p: Path) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::Paradigm` | type | `lib/compiler/semcore.myc:211` | `type Paradigm = PBinary \| PTernary \| PDense \| PVsa` | Paradigm: mirrors ast.myc::Paradigm (FLAG-ast-4 P-prefix, reused verbatim). | Empirical/Declared |
| `compiler.semcore::Paradigm::PBinary` | ctor | `lib/compiler/semcore.myc:211` | `PBinary` | — | Empirical/Declared |
| `compiler.semcore::Paradigm::PDense` | ctor | `lib/compiler/semcore.myc:211` | `PDense` | — | Empirical/Declared |
| `compiler.semcore::Paradigm::PTernary` | ctor | `lib/compiler/semcore.myc:211` | `PTernary` | — | Empirical/Declared |
| `compiler.semcore::Paradigm::PVsa` | ctor | `lib/compiler/semcore.myc:211` | `PVsa` | — | Empirical/Declared |
| `compiler.semcore::Scalar` | type | `lib/compiler/semcore.myc:214` | `type Scalar = SF16 \| SBf16 \| SF32 \| SF64` | Scalar: mirrors ast.myc::Scalar (FLAG-ast-4 S-prefix, reused verbatim). | Empirical/Declared |
| `compiler.semcore::Scalar::SBf16` | ctor | `lib/compiler/semcore.myc:214` | `SBf16` | — | Empirical/Declared |
| `compiler.semcore::Scalar::SF16` | ctor | `lib/compiler/semcore.myc:214` | `SF16` | — | Empirical/Declared |
| `compiler.semcore::Scalar::SF32` | ctor | `lib/compiler/semcore.myc:214` | `SF32` | — | Empirical/Declared |
| `compiler.semcore::Scalar::SF64` | ctor | `lib/compiler/semcore.myc:214` | `SF64` | — | Empirical/Declared |
| `compiler.semcore::Sparsity` | type | `lib/compiler/semcore.myc:217` | `type Sparsity = SpDense \| SpSparse(Binary{32})` | Sparsity: mirrors ast.myc::Sparsity (FLAG-ast-4 Sp-prefix, reused verbatim). | Empirical/Declared |
| `compiler.semcore::Sparsity::SpDense` | ctor | `lib/compiler/semcore.myc:217` | `SpDense` | — | Empirical/Declared |
| `compiler.semcore::Sparsity::SpSparse` | ctor | `lib/compiler/semcore.myc:217` | `SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::AmbientParams` | type | `lib/compiler/semcore.myc:220` | `type AmbientParams = APSize(Binary{32}) \| APDense(Binary{32}, Scalar) \| APVsa(Bytes, Binary{32}, Sparsity)` | AmbientParams: mirrors ast.myc::AmbientParams (FLAG-ast-4/5 AP-prefix, reused verbatim). | Empirical/Declared |
| `compiler.semcore::AmbientParams::APSize` | ctor | `lib/compiler/semcore.myc:221` | `APSize(Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::AmbientParams::APDense` | ctor | `lib/compiler/semcore.myc:222` | `APDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.semcore::AmbientParams::APVsa` | ctor | `lib/compiler/semcore.myc:223` | `APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.semcore::Strength` | type | `lib/compiler/semcore.myc:227` | `type Strength = GExact \| GProven \| GEmpirical \| GDeclared` | Strength: mirrors ast.myc::Strength (the guarantee-lattice tag; FLAG-ast-4 G-prefix, reused verbatim). Core to grade.rs's port — rank/meet/satisfies are all needed. | Empirical/Declared |
| `compiler.semcore::Strength::GDeclared` | ctor | `lib/compiler/semcore.myc:227` | `GDeclared` | — | Empirical/Declared |
| `compiler.semcore::Strength::GEmpirical` | ctor | `lib/compiler/semcore.myc:227` | `GEmpirical` | — | Empirical/Declared |
| `compiler.semcore::Strength::GExact` | ctor | `lib/compiler/semcore.myc:227` | `GExact` | — | Empirical/Declared |
| `compiler.semcore::Strength::GProven` | ctor | `lib/compiler/semcore.myc:227` | `GProven` | — | Empirical/Declared |
| `compiler.semcore::strength_rank` | fn | `lib/compiler/semcore.myc:229` | `fn strength_rank(s: Strength) => Binary{8}` | — | Empirical/Declared |
| `compiler.semcore::strength_meet` | fn | `lib/compiler/semcore.myc:238` | `fn strength_meet(a: Strength, b: Strength) => Strength` | strength_meet: the weaker (less-trusted) of the two grades (mirrors ast.myc::strength_meet). | Empirical/Declared |
| `compiler.semcore::strength_satisfies` | fn | `lib/compiler/semcore.myc:245` | `fn strength_satisfies(actual: Strength, demand: Strength) => Bool` | strength_satisfies: `actual.rank() >= demand.rank()` (mirrors ast.myc::strength_satisfies). | Empirical/Declared |
| `compiler.semcore::WidthRef` | type | `lib/compiler/semcore.myc:253` | `type WidthRef = WLit(Binary{32}) \| WName(Bytes)` | WidthRef: mirrors ast.myc::WidthRef (a SURFACE width slot — distinct from checkty's own `Width` below, FLAG-semcore-2). | Empirical/Declared |
| `compiler.semcore::WidthRef::WLit` | ctor | `lib/compiler/semcore.myc:253` | `WLit(Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::WidthRef::WName` | ctor | `lib/compiler/semcore.myc:253` | `WName(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::ParamKind` | type | `lib/compiler/semcore.myc:256` | `type ParamKind = PkType \| PkWidth` | ParamKind: mirrors ast.myc::ParamKind. | Empirical/Declared |
| `compiler.semcore::ParamKind::PkType` | ctor | `lib/compiler/semcore.myc:256` | `PkType` | — | Empirical/Declared |
| `compiler.semcore::ParamKind::PkWidth` | ctor | `lib/compiler/semcore.myc:256` | `PkWidth` | — | Empirical/Declared |
| `compiler.semcore::TraitRef` | type | `lib/compiler/semcore.myc:260` | `type TraitRef = TRf(Bytes, Vec[TypeRef])` | TraitRef: mirrors ast.myc::TraitRef (a bound-position trait reference; field only, unused accessors dropped per FLAG-semcore-1). | Empirical/Declared |
| `compiler.semcore::TraitRef::TRf` | ctor | `lib/compiler/semcore.myc:260` | `TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.semcore::TypeParam` | type | `lib/compiler/semcore.myc:263` | `type TypeParam = TP(Bytes, ParamKind, Vec[TraitRef])` | TypeParam: mirrors ast.myc::TypeParam (field only, unused accessors dropped). | Empirical/Declared |
| `compiler.semcore::TypeParam::TP` | ctor | `lib/compiler/semcore.myc:263` | `TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.semcore::EffectBudget` | type | `lib/compiler/semcore.myc:266` | `type EffectBudget = EB(Bytes, Binary{64})` | EffectBudget: mirrors ast.myc::EffectBudget (FLAG-ast-6; field only). | Empirical/Declared |
| `compiler.semcore::EffectBudget::EB` | ctor | `lib/compiler/semcore.myc:266` | `EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.semcore::FnSig` | type | `lib/compiler/semcore.myc:270` | `type FnSig = FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | FnSig: mirrors ast.myc::FnSig field-for-field. Only the 3 accessors grade.rs actually calls are kept (name, value_params, ret) — fnsig_params/fnsig_effects/fnsig_effect_budgets are unused here. | Empirical/Declared |
| `compiler.semcore::FnSig::FS` | ctor | `lib/compiler/semcore.myc:270` | `FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.semcore::fnsig_name` | fn | `lib/compiler/semcore.myc:272` | `fn fnsig_name(s: FnSig) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::fnsig_value_params` | fn | `lib/compiler/semcore.myc:275` | `fn fnsig_value_params(s: FnSig) => Vec[Param]` | — | Empirical/Declared |
| `compiler.semcore::fnsig_ret` | fn | `lib/compiler/semcore.myc:278` | `fn fnsig_ret(s: FnSig) => TypeRef` | — | Empirical/Declared |
| `compiler.semcore::Param` | type | `lib/compiler/semcore.myc:282` | `type Param = Prm(Bytes, TypeRef)` | Param: mirrors ast.myc::Param (a value parameter `name: type`). | Empirical/Declared |
| `compiler.semcore::Param::Prm` | ctor | `lib/compiler/semcore.myc:282` | `Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.semcore::param_name` | fn | `lib/compiler/semcore.myc:284` | `fn param_name(p: Param) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::param_ty` | fn | `lib/compiler/semcore.myc:287` | `fn param_ty(p: Param) => TypeRef` | — | Empirical/Declared |
| `compiler.semcore::TypeRef` | type | `lib/compiler/semcore.myc:293` | `type TypeRef = TR(BaseType, Option[Strength])` | TypeRef / BaseType: mirror ast.myc::TypeRef / ast.myc::BaseType (mutually recursive — the checker's shell-first type registration resolves this regardless of declaration order, per ast.myc's own header note). Only `typeref_guarantee` is kept (grade never inspects the base). | Empirical/Declared |
| `compiler.semcore::TypeRef::TR` | ctor | `lib/compiler/semcore.myc:293` | `TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.semcore::typeref_guarantee` | fn | `lib/compiler/semcore.myc:295` | `fn typeref_guarantee(t: TypeRef) => Option[Strength]` | — | Empirical/Declared |
| `compiler.semcore::BaseType` | type | `lib/compiler/semcore.myc:301` | `type BaseType = KwBinary(WidthRef) \| KwTernary(WidthRef) \| KwDense(Binary{32}, Scalar) \| Vsa(Bytes, Binary{32}, Sparsity) \| KwSubstrate(Bytes) \| KwSeq(TypeRef, Binary{32}) \| KwBytes \| KwFloat \| Named(Bytes, Vec[TypeRef]) \| Ambient(AmbientParams) \| FnArrow(TypeRef, TypeRef) \| Tuple(Vec[TypeRef])` | BaseType: mirrors ast.myc::BaseType verbatim (FLAG-ast-4/5 renames carried over unchanged — `Vsa`/`KwSubstrate` here are ast.myc's OWN surface spellings, distinct from this file's NEW `TyVsa`/`TySubstrate` checked-type family, FLAG-semcore-2). | Empirical/Declared |
| `compiler.semcore::BaseType::KwBinary` | ctor | `lib/compiler/semcore.myc:302` | `KwBinary(WidthRef)` | — | Empirical/Declared |
| `compiler.semcore::BaseType::KwTernary` | ctor | `lib/compiler/semcore.myc:303` | `KwTernary(WidthRef)` | — | Empirical/Declared |
| `compiler.semcore::BaseType::KwDense` | ctor | `lib/compiler/semcore.myc:304` | `KwDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.semcore::BaseType::Vsa` | ctor | `lib/compiler/semcore.myc:305` | `Vsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.semcore::BaseType::KwSubstrate` | ctor | `lib/compiler/semcore.myc:306` | `KwSubstrate(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::BaseType::KwSeq` | ctor | `lib/compiler/semcore.myc:307` | `KwSeq(TypeRef, Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::BaseType::KwBytes` | ctor | `lib/compiler/semcore.myc:308` | `KwBytes` | — | Empirical/Declared |
| `compiler.semcore::BaseType::KwFloat` | ctor | `lib/compiler/semcore.myc:309` | `KwFloat` | — | Empirical/Declared |
| `compiler.semcore::BaseType::Named` | ctor | `lib/compiler/semcore.myc:310` | `Named(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.semcore::BaseType::Ambient` | ctor | `lib/compiler/semcore.myc:311` | `Ambient(AmbientParams)` | — | Empirical/Declared |
| `compiler.semcore::BaseType::FnArrow` | ctor | `lib/compiler/semcore.myc:312` | `FnArrow(TypeRef, TypeRef)` | — | Empirical/Declared |
| `compiler.semcore::BaseType::Tuple` | ctor | `lib/compiler/semcore.myc:313` | `Tuple(Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.semcore::Ctor` | type | `lib/compiler/semcore.myc:319` | `type Ctor = Ct(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.semcore::Ctor::Ct` | ctor | `lib/compiler/semcore.myc:319` | `Ct(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.semcore::ct_name` | fn | `lib/compiler/semcore.myc:321` | `fn ct_name(c: Ctor) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::ct_fields` | fn | `lib/compiler/semcore.myc:324` | `fn ct_fields(c: Ctor) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.semcore::TypeDecl` | type | `lib/compiler/semcore.myc:328` | `type TypeDecl = TD(Vis, Bytes, Vec[Bytes], Vec[Ctor])` | TypeDecl: mirrors ast.rs::TypeDecl field-for-field (vis, name, params, ctors — declaration order). | Empirical/Declared |
| `compiler.semcore::TypeDecl::TD` | ctor | `lib/compiler/semcore.myc:328` | `TD(Vis, Bytes, Vec[Bytes], Vec[Ctor])` | — | Empirical/Declared |
| `compiler.semcore::td_name` | fn | `lib/compiler/semcore.myc:330` | `fn td_name(t: TypeDecl) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::td_params` | fn | `lib/compiler/semcore.myc:333` | `fn td_params(t: TypeDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::td_ctors` | fn | `lib/compiler/semcore.myc:336` | `fn td_ctors(t: TypeDecl) => Vec[Ctor]` | — | Empirical/Declared |
| `compiler.semcore::ExecutionMode` | type | `lib/compiler/semcore.myc:340` | `type ExecutionMode = Interpreted \| Compiled` | ExecutionMode: mirrors ast.myc::ExecutionMode (a field of `FnDecl`; unused by grade otherwise). | Empirical/Declared |
| `compiler.semcore::ExecutionMode::Compiled` | ctor | `lib/compiler/semcore.myc:340` | `Compiled` | — | Empirical/Declared |
| `compiler.semcore::ExecutionMode::Interpreted` | ctor | `lib/compiler/semcore.myc:340` | `Interpreted` | — | Empirical/Declared |
| `compiler.semcore::FnDecl` | type | `lib/compiler/semcore.myc:343` | `type FnDecl = FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | FnDecl: mirrors ast.myc::FnDecl field-for-field. Only `sig`/`body` accessors are kept. | Empirical/Declared |
| `compiler.semcore::FnDecl::FD` | ctor | `lib/compiler/semcore.myc:343` | `FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.semcore::fndecl_sig` | fn | `lib/compiler/semcore.myc:345` | `fn fndecl_sig(f: FnDecl) => FnSig` | — | Empirical/Declared |
| `compiler.semcore::fndecl_body` | fn | `lib/compiler/semcore.myc:348` | `fn fndecl_body(f: FnDecl) => Expr` | — | Empirical/Declared |
| `compiler.semcore::Literal` | type | `lib/compiler/semcore.myc:353` | `type Literal = Bin(Bytes) \| Trit(Bytes) \| Int(Binary{64}) \| AmbientInt(Paradigm, Binary{64}) \| List(Vec[Expr]) \| LBytes(Bytes) \| Str(Bytes) \| LFloat(Bytes)` | Literal: mirrors ast.myc::Literal verbatim (FLAG-ast-4/5 renames carried over — `LBytes`/ `LFloat`, FLAG-ast-8: no `#[non_exhaustive]` analog). | Empirical/Declared |
| `compiler.semcore::Literal::Bin` | ctor | `lib/compiler/semcore.myc:354` | `Bin(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Literal::Trit` | ctor | `lib/compiler/semcore.myc:355` | `Trit(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Literal::Int` | ctor | `lib/compiler/semcore.myc:356` | `Int(Binary{64})` | — | Empirical/Declared |
| `compiler.semcore::Literal::AmbientInt` | ctor | `lib/compiler/semcore.myc:357` | `AmbientInt(Paradigm, Binary{64})` | — | Empirical/Declared |
| `compiler.semcore::Literal::List` | ctor | `lib/compiler/semcore.myc:358` | `List(Vec[Expr])` | — | Empirical/Declared |
| `compiler.semcore::Literal::LBytes` | ctor | `lib/compiler/semcore.myc:359` | `LBytes(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Literal::Str` | ctor | `lib/compiler/semcore.myc:360` | `Str(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Literal::LFloat` | ctor | `lib/compiler/semcore.myc:361` | `LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Pattern` | type | `lib/compiler/semcore.myc:364` | `type Pattern = PWildcard \| PLit(Literal) \| PCtor(Bytes, Vec[Pattern]) \| PIdent(Bytes) \| PTuple(Vec[Pattern]) \| POr(Vec[Pattern])` | Pattern: mirrors ast.myc::Pattern verbatim (FLAG-ast-5 P-prefix throughout). | Empirical/Declared |
| `compiler.semcore::Pattern::PWildcard` | ctor | `lib/compiler/semcore.myc:365` | `PWildcard` | — | Empirical/Declared |
| `compiler.semcore::Pattern::PLit` | ctor | `lib/compiler/semcore.myc:366` | `PLit(Literal)` | — | Empirical/Declared |
| `compiler.semcore::Pattern::PCtor` | ctor | `lib/compiler/semcore.myc:367` | `PCtor(Bytes, Vec[Pattern])` | — | Empirical/Declared |
| `compiler.semcore::Pattern::PIdent` | ctor | `lib/compiler/semcore.myc:368` | `PIdent(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Pattern::PTuple` | ctor | `lib/compiler/semcore.myc:369` | `PTuple(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.semcore::Pattern::POr` | ctor | `lib/compiler/semcore.myc:370` | `POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.semcore::Arm` | type | `lib/compiler/semcore.myc:373` | `type Arm = Ar(Pattern, Expr)` | Arm: mirrors ast.myc::Arm (one `match` arm). | Empirical/Declared |
| `compiler.semcore::Arm::Ar` | ctor | `lib/compiler/semcore.myc:373` | `Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.semcore::arm_pattern` | fn | `lib/compiler/semcore.myc:375` | `fn arm_pattern(a: Arm) => Pattern` | — | Empirical/Declared |
| `compiler.semcore::arm_body` | fn | `lib/compiler/semcore.myc:378` | `fn arm_body(a: Arm) => Expr` | — | Empirical/Declared |
| `compiler.semcore::Hypha` | type | `lib/compiler/semcore.myc:382` | `type Hypha = Hy(Option[Expr], Expr)` | Hypha: mirrors ast.myc::Hypha (only `body` accessor kept — grade never inspects `forage`). | Empirical/Declared |
| `compiler.semcore::Hypha::Hy` | ctor | `lib/compiler/semcore.myc:382` | `Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.semcore::hypha_body` | fn | `lib/compiler/semcore.myc:384` | `fn hypha_body(h: Hypha) => Expr` | — | Empirical/Declared |
| `compiler.semcore::Expr` | type | `lib/compiler/semcore.myc:388` | `type Expr = Let(Bytes, Option[TypeRef], Expr, Expr) \| If(Expr, Expr, Expr) \| Match(Expr, Vec[Arm]) \| For(Bytes, Expr, Bytes, Expr, Expr) \| Swap(Expr, TypeRef, Path) \| WithParadigm(Paradigm, Expr) \| Wild(Expr) \| Spore(Expr) \| Consume(Expr) \| Colony(Vec[Hypha]) \| Lambda(Vec[Param], Expr) \| App(Expr, Vec[Expr]) \| Fuse(Expr, Expr) \| Reclaim(Expr, Expr) \| Path(Path) \| Lit(Literal) \| Ascribe(Expr, TypeRef) \| TupleLit(Vec[Expr])` | Expr: mirrors ast.myc::Expr verbatim, field-for-field, same variant order. | Empirical/Declared |
| `compiler.semcore::Expr::Let` | ctor | `lib/compiler/semcore.myc:389` | `Let(Bytes, Option[TypeRef], Expr, Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::If` | ctor | `lib/compiler/semcore.myc:390` | `If(Expr, Expr, Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Match` | ctor | `lib/compiler/semcore.myc:391` | `Match(Expr, Vec[Arm])` | — | Empirical/Declared |
| `compiler.semcore::Expr::For` | ctor | `lib/compiler/semcore.myc:392` | `For(Bytes, Expr, Bytes, Expr, Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Path` | ctor | `lib/compiler/semcore.myc:393` | `Path(Path)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Swap` | ctor | `lib/compiler/semcore.myc:393` | `Swap(Expr, TypeRef, Path)` | — | Empirical/Declared |
| `compiler.semcore::Expr::WithParadigm` | ctor | `lib/compiler/semcore.myc:394` | `WithParadigm(Paradigm, Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Wild` | ctor | `lib/compiler/semcore.myc:395` | `Wild(Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Spore` | ctor | `lib/compiler/semcore.myc:396` | `Spore(Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Consume` | ctor | `lib/compiler/semcore.myc:397` | `Consume(Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Colony` | ctor | `lib/compiler/semcore.myc:398` | `Colony(Vec[Hypha])` | — | Empirical/Declared |
| `compiler.semcore::Expr::Lambda` | ctor | `lib/compiler/semcore.myc:399` | `Lambda(Vec[Param], Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::App` | ctor | `lib/compiler/semcore.myc:400` | `App(Expr, Vec[Expr])` | — | Empirical/Declared |
| `compiler.semcore::Expr::Fuse` | ctor | `lib/compiler/semcore.myc:401` | `Fuse(Expr, Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Reclaim` | ctor | `lib/compiler/semcore.myc:402` | `Reclaim(Expr, Expr)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Lit` | ctor | `lib/compiler/semcore.myc:404` | `Lit(Literal)` | — | Empirical/Declared |
| `compiler.semcore::Expr::Ascribe` | ctor | `lib/compiler/semcore.myc:405` | `Ascribe(Expr, TypeRef)` | — | Empirical/Declared |
| `compiler.semcore::Expr::TupleLit` | ctor | `lib/compiler/semcore.myc:406` | `TupleLit(Vec[Expr])` | — | Empirical/Declared |
| `compiler.semcore::zero32` | fn | `lib/compiler/semcore.myc:409` | `fn zero32() => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::one32` | fn | `lib/compiler/semcore.myc:410` | `fn one32() => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::max_semcore_depth` | fn | `lib/compiler/semcore.myc:414` | `fn max_semcore_depth() => Binary{32}` | max_semcore_depth: SAME 4096 ceiling as checkty::MAX_CHECK_DEPTH / parse.myc::max_expr_depth (FLAG-semcore-3). | Empirical/Declared |
| `compiler.semcore::enter_depth` | fn | `lib/compiler/semcore.myc:418` | `fn enter_depth(depth: Binary{32}) => Result[Binary{32}, Bytes]` | enter_depth: charge one level against the shared budget; `Err` on overflow (never a panic — the parse.myc FLAG-parse-7 convention, restated for this nodule's three budgeted recursions). | Empirical/Declared |
| `compiler.semcore::names_contains` | fn | `lib/compiler/semcore.myc:427` | `fn names_contains(names: Vec[Bytes], n: Bytes) => Bool` | names_contains: linear membership check over a `Vec[Bytes]` (used throughout for dedup / signature-completeness checks — the FLAG-semcore-4 assoc-list convention). | Empirical/Declared |
| `compiler.semcore::Width` | type | `lib/compiler/semcore.myc:437` | `type Width = WdLit(Binary{32}) \| WdVar(Bytes)` | Width: mirrors checkty.rs::Width (a `Binary{n}`/`Ternary{m}` width argument — concrete literal or abstract width variable). | Empirical/Declared |
| `compiler.semcore::Width::WdLit` | ctor | `lib/compiler/semcore.myc:437` | `WdLit(Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::Width::WdVar` | ctor | `lib/compiler/semcore.myc:437` | `WdVar(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Ty` | type | `lib/compiler/semcore.myc:441` | `type Ty = TyBinary(Width) \| TyTernary(Width) \| TyDense(Binary{32}, Scalar) \| TyVsa(Bytes, Binary{32}, Sparsity) \| TyData(Bytes, Vec[Ty]) \| TySubstrate(Bytes) \| TySeq(Ty, Binary{32}) \| TyBytes \| TyFloat \| TyVar(Bytes) \| TyFn(Ty, Ty)` | Ty: mirrors checkty.rs::Ty field-for-field (11 variants; `Box<Ty>` fields carry over as plain `Ty` per ast.myc's own shell-first-registration recursion note — no wrapper needed). | Empirical/Declared |
| `compiler.semcore::Ty::TyBinary` | ctor | `lib/compiler/semcore.myc:442` | `TyBinary(Width)` | — | Empirical/Declared |
| `compiler.semcore::Ty::TyTernary` | ctor | `lib/compiler/semcore.myc:443` | `TyTernary(Width)` | — | Empirical/Declared |
| `compiler.semcore::Ty::TyDense` | ctor | `lib/compiler/semcore.myc:444` | `TyDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.semcore::Ty::TyVsa` | ctor | `lib/compiler/semcore.myc:445` | `TyVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.semcore::Ty::TyData` | ctor | `lib/compiler/semcore.myc:446` | `TyData(Bytes, Vec[Ty])` | — | Empirical/Declared |
| `compiler.semcore::Ty::TySubstrate` | ctor | `lib/compiler/semcore.myc:447` | `TySubstrate(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Ty::TySeq` | ctor | `lib/compiler/semcore.myc:448` | `TySeq(Ty, Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::Ty::TyBytes` | ctor | `lib/compiler/semcore.myc:449` | `TyBytes` | — | Empirical/Declared |
| `compiler.semcore::Ty::TyFloat` | ctor | `lib/compiler/semcore.myc:450` | `TyFloat` | — | Empirical/Declared |
| `compiler.semcore::Ty::TyVar` | ctor | `lib/compiler/semcore.myc:451` | `TyVar(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Ty::TyFn` | ctor | `lib/compiler/semcore.myc:452` | `TyFn(Ty, Ty)` | — | Empirical/Declared |
| `compiler.semcore::CtorInfo` | type | `lib/compiler/semcore.myc:455` | `type CtorInfo = CI(Bytes, Vec[Ty])` | CtorInfo: mirrors checkty.rs::CtorInfo (name, field types). | Empirical/Declared |
| `compiler.semcore::CtorInfo::CI` | ctor | `lib/compiler/semcore.myc:455` | `CI(Bytes, Vec[Ty])` | — | Empirical/Declared |
| `compiler.semcore::ci_name` | fn | `lib/compiler/semcore.myc:457` | `fn ci_name(c: CtorInfo) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::ci_fields` | fn | `lib/compiler/semcore.myc:460` | `fn ci_fields(c: CtorInfo) => Vec[Ty]` | — | Empirical/Declared |
| `compiler.semcore::DataInfo` | type | `lib/compiler/semcore.myc:464` | `type DataInfo = DI(Bytes, Vec[Bytes], Vec[CtorInfo])` | DataInfo: mirrors checkty.rs::DataInfo (name, type params, ctors in declaration order). | Empirical/Declared |
| `compiler.semcore::DataInfo::DI` | ctor | `lib/compiler/semcore.myc:464` | `DI(Bytes, Vec[Bytes], Vec[CtorInfo])` | — | Empirical/Declared |
| `compiler.semcore::di_name` | fn | `lib/compiler/semcore.myc:466` | `fn di_name(d: DataInfo) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::di_ctors` | fn | `lib/compiler/semcore.myc:469` | `fn di_ctors(d: DataInfo) => Vec[CtorInfo]` | — | Empirical/Declared |
| `compiler.semcore::types_lookup` | fn | `lib/compiler/semcore.myc:474` | `fn types_lookup(types: Vec[DataInfo], name: Bytes) => Option[DataInfo]` | types_lookup: the FLAG-semcore-4 ordered-assoc-list lookup standing in for `BTreeMap<String, DataInfo>`. | Empirical/Declared |
| `compiler.semcore::Pat` | type | `lib/compiler/semcore.myc:486` | `type Pat = MpWild \| MpCtor(Bytes, Vec[Pat]) \| MpLit(Bytes)` | Pat: mirrors usefulness.rs::Pat (the NORMALIZED matrix pattern — FLAG-semcore-2 Mp-prefix). | Empirical/Declared |
| `compiler.semcore::Pat::MpCtor` | ctor | `lib/compiler/semcore.myc:486` | `MpCtor(Bytes, Vec[Pat])` | — | Empirical/Declared |
| `compiler.semcore::Pat::MpLit` | ctor | `lib/compiler/semcore.myc:486` | `MpLit(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Pat::MpWild` | ctor | `lib/compiler/semcore.myc:486` | `MpWild` | — | Empirical/Declared |
| `compiler.semcore::pat_append` | fn | `lib/compiler/semcore.myc:490` | `fn pat_append(a: Vec[Pat], b: Vec[Pat]) => Vec[Pat]` | pat_append / pat_len: small list helpers (non-tail, bounded by ONE row's arity — a handful of fields, never source length; mirrors nodule.myc::join_segs's count-bounded-recursion precedent). | Empirical/Declared |
| `compiler.semcore::pat_len` | fn | `lib/compiler/semcore.myc:493` | `fn pat_len(v: Vec[Pat]) => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::ty_len` | fn | `lib/compiler/semcore.myc:499` | `fn ty_len(v: Vec[Ty]) => Binary{32}` | ty_len: the arity-counting twin of `pat_len`, over a constructor's OWN field-type list (`Vec[Ty]`, not `Vec[Pat]` — a distinct monomorphic length fn since this language's total-fn discipline rules out a single generic one here, FLAG-semcore-5's KISS/YAGNI rationale extended). | Empirical/Declared |
| `compiler.semcore::wild_n` | fn | `lib/compiler/semcore.myc:503` | `fn wild_n(n: Binary{32}) => Vec[Pat]` | wild_n: `n` fresh wildcards (bounded by a constructor's own arity). | Empirical/Declared |
| `compiler.semcore::ty_append` | fn | `lib/compiler/semcore.myc:510` | `fn ty_append(a: Vec[Ty], b: Vec[Ty]) => Vec[Ty]` | ty_append: mirrors the Rust `ct2.extend_from_slice(...)` column-type splicing. | Empirical/Declared |
| `compiler.semcore::ct_head` | fn | `lib/compiler/semcore.myc:516` | `fn ct_head(ts: Vec[Ty]) => Ty` | ct_head / ct_tail: single-level column-type accessors (the `Nil` arm is an internal-invariant edge — never hit by a well-formed call where `col_types` is parallel to the query — a harmless sentinel value, not a silent behavior change to any reachable case). | Empirical/Declared |
| `compiler.semcore::ct_tail` | fn | `lib/compiler/semcore.myc:519` | `fn ct_tail(ts: Vec[Ty]) => Vec[Ty]` | — | Empirical/Declared |
| `compiler.semcore::signature` | fn | `lib/compiler/semcore.myc:525` | `fn signature(ty: Ty, types: Vec[DataInfo]) => Option[DataInfo]` | signature: the finite constructor signature of `ty`, or `None` if its domain is open (`Binary`/`Ternary` — mirrors usefulness.rs::signature; the trailing wildcard is the established `_ => …` idiom over a big-variant sum type, parse.myc's own `Tok` classifiers). | Empirical/Declared |
| `compiler.semcore::ctorinfo_fields_by_name` | fn | `lib/compiler/semcore.myc:530` | `fn ctorinfo_fields_by_name(ctors: Vec[CtorInfo], c: Bytes) => Vec[Ty]` | ctorinfo_fields_by_name / ctor_fields_of: the field types of constructor `c` (mirrors usefulness.rs::ctor_fields; empty if not found — a well-typed matrix never misses). | Empirical/Declared |
| `compiler.semcore::ctor_fields_of` | fn | `lib/compiler/semcore.myc:539` | `fn ctor_fields_of(ty: Ty, c: Bytes, types: Vec[DataInfo]) => Vec[Ty]` | — | Empirical/Declared |
| `compiler.semcore::specialize_ctor_pat` | fn | `lib/compiler/semcore.myc:548` | `fn specialize_ctor_pat(matrix: Vec[Vec[Pat]], c: Bytes, a: Binary{32}) => Vec[Vec[Pat]]` | specialize_ctor_pat / specialize_lit_pat: mirror usefulness.rs::specialize_ctor/specialize_lit over a BARE `Vec[Pat]` matrix (FLAG-semcore-5 — no shared `SpecializeRow` trait; this is the `useful`-side copy, `specialize_ctor_row`/`specialize_lit_row` below is the `compile`-side one). | Empirical/Declared |
| `compiler.semcore::specialize_lit_pat` | fn | `lib/compiler/semcore.myc:564` | `fn specialize_lit_pat(matrix: Vec[Vec[Pat]], k: Bytes) => Vec[Vec[Pat]]` | — | Empirical/Declared |
| `compiler.semcore::default_matrix` | fn | `lib/compiler/semcore.myc:581` | `fn default_matrix(matrix: Vec[Vec[Pat]]) => Vec[Vec[Pat]]` | default_matrix: `D(P)` — rows headed by a wildcard, leading column dropped. | Empirical/Declared |
| `compiler.semcore::head_ctors` | fn | `lib/compiler/semcore.myc:595` | `fn head_ctors(matrix: Vec[Vec[Pat]]) => Vec[Bytes]` | head_ctors: the (deduped) constructor names appearing in the matrix's first column. | Empirical/Declared |
| `compiler.semcore::ctors_all_present` | fn | `lib/compiler/semcore.myc:611` | `fn ctors_all_present(ctors: Vec[CtorInfo], present: Vec[Bytes]) => Bool` | ctors_all_present / find_missing_ctor: shared by usefulness AND decision's completeness checks. | Empirical/Declared |
| `compiler.semcore::find_missing_ctor` | fn | `lib/compiler/semcore.myc:620` | `fn find_missing_ctor(ctors: Vec[CtorInfo], present: Vec[Bytes]) => Option[CtorInfo]` | — | Empirical/Declared |
| `compiler.semcore::missing_head` | fn | `lib/compiler/semcore.myc:631` | `fn missing_head(m: Option[CtorInfo]) => Pat` | missing_head: the witness head for an incomplete column (mirrors usefulness.rs's inline `missing.map_or(Pat::Wild, ...)`). | Empirical/Declared |
| `compiler.semcore::pat_split_at` | fn | `lib/compiler/semcore.myc:635` | `fn pat_split_at(w: Vec[Pat], n: Binary{32}) => Pair[Vec[Pat], Vec[Pat]]` | pat_split_at: split a witness vector at `n` (mirrors usefulness.rs's `w.split_off(a)`). | Empirical/Declared |
| `compiler.semcore::rebuild_ctor` | fn | `lib/compiler/semcore.myc:648` | `fn rebuild_ctor(c: Bytes, a: Binary{32}, w: Vec[Pat]) => Vec[Pat]` | rebuild_ctor / prepend: re-fold a witness whose first `a` elements are constructor `c`'s sub-witnesses (mirrors usefulness.rs verbatim). | Empirical/Declared |
| `compiler.semcore::prepend` | fn | `lib/compiler/semcore.myc:651` | `fn prepend(head: Pat, rest: Vec[Pat]) => Vec[Pat]` | — | Empirical/Declared |
| `compiler.semcore::matrix_is_empty_to_witness` | fn | `lib/compiler/semcore.myc:656` | `fn matrix_is_empty_to_witness(matrix: Vec[Vec[Pat]]) => Option[Vec[Pat]]` | matrix_is_empty_to_witness: the `q.is_empty()` base case's witness (`Some([])` iff no row remains). | Empirical/Declared |
| `compiler.semcore::useful` | fn | `lib/compiler/semcore.myc:660` | `fn useful(types: Vec[DataInfo], matrix: Vec[Vec[Pat]], q: Vec[Pat], col_types: Vec[Ty]) => Result[Option[Vec[Pat]], Bytes]` | useful: `U(P, q)` entry point — a fresh per-query budget (mirrors usefulness.rs::useful). | Empirical/Declared |
| `compiler.semcore::useful_budgeted` | fn | `lib/compiler/semcore.myc:665` | `fn useful_budgeted(depth: Binary{32}, types: Vec[DataInfo], matrix: Vec[Vec[Pat]], q: Vec[Pat], col_types: Vec[Ty]) => Result[Option[Vec[Pat]], Bytes]` | useful_budgeted: the budget-charged recursion (mirrors usefulness.rs::useful_budgeted). | Empirical/Declared |
| `compiler.semcore::useful_ctor` | fn | `lib/compiler/semcore.myc:679` | `fn useful_ctor(depth: Binary{32}, types: Vec[DataInfo], matrix: Vec[Vec[Pat]], qrest: Vec[Pat], col_types: Vec[Ty], c: Bytes, subs: Vec[Pat]) => Result[Option[Vec[Pat]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::useful_lit` | fn | `lib/compiler/semcore.myc:693` | `fn useful_lit(depth: Binary{32}, types: Vec[DataInfo], matrix: Vec[Vec[Pat]], qrest: Vec[Pat], col_types: Vec[Ty], k: Bytes) => Result[Option[Vec[Pat]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::useful_wild` | fn | `lib/compiler/semcore.myc:704` | `fn useful_wild(depth: Binary{32}, types: Vec[DataInfo], matrix: Vec[Vec[Pat]], qrest: Vec[Pat], col_types: Vec[Ty]) => Result[Option[Vec[Pat]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::useful_wild_open` | fn | `lib/compiler/semcore.myc:716` | `fn useful_wild_open(depth: Binary{32}, types: Vec[DataInfo], matrix: Vec[Vec[Pat]], qrest: Vec[Pat], col_types: Vec[Ty]) => Result[Option[Vec[Pat]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::useful_wild_incomplete` | fn | `lib/compiler/semcore.myc:727` | `fn useful_wild_incomplete(depth: Binary{32}, types: Vec[DataInfo], matrix: Vec[Vec[Pat]], qrest: Vec[Pat], col_types: Vec[Ty], d: DataInfo, present: Vec[Bytes]) => Result[Option[Vec[Pat]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::useful_wild_complete` | fn | `lib/compiler/semcore.myc:741` | `fn useful_wild_complete(depth: Binary{32}, types: Vec[DataInfo], matrix: Vec[Vec[Pat]], qrest: Vec[Pat], col_types: Vec[Ty], ctors: Vec[CtorInfo]) => Result[Option[Vec[Pat]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::render` | fn | `lib/compiler/semcore.myc:762` | `fn render(p: Pat) => Bytes` | render: render a witness pattern for a diagnostic (mirrors usefulness.rs::render; the b:/t: literal-key rewrite is SKIPPED here — cosmetic-only, not exercised by this increment's differential, which compares verdict SHAPE, not rendered text). | Empirical/Declared |
| `compiler.semcore::render_list` | fn | `lib/compiler/semcore.myc:772` | `fn render_list(subs: Vec[Pat]) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::Head` | type | `lib/compiler/semcore.myc:784` | `type Head = HdCtor(Bytes, Binary{32}) \| HdLit(Bytes)` | Head: mirrors decision.rs::Head (FLAG-semcore-2 Hd-prefix). | Empirical/Declared |
| `compiler.semcore::Head::HdCtor` | ctor | `lib/compiler/semcore.myc:784` | `HdCtor(Bytes, Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::Head::HdLit` | ctor | `lib/compiler/semcore.myc:784` | `HdLit(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Tree` | type | `lib/compiler/semcore.myc:788` | `type Tree = Leaf(Binary{32}) \| Fail \| Switch(Vec[Binary{32}], Vec[Pair[Head, Tree]], Option[Tree])` | Tree: mirrors decision.rs::Tree (no Box needed — recursion note above; `Occurrence` inlined as `Vec[Binary{32}]`, FLAG-semcore-7). | Empirical/Declared |
| `compiler.semcore::Tree::Fail` | ctor | `lib/compiler/semcore.myc:788` | `Fail` | — | Empirical/Declared |
| `compiler.semcore::Tree::Leaf` | ctor | `lib/compiler/semcore.myc:788` | `Leaf(Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::Tree::Switch` | ctor | `lib/compiler/semcore.myc:788` | `Switch(Vec[Binary{32}], Vec[Pair[Head, Tree]], Option[Tree])` | — | Empirical/Declared |
| `compiler.semcore::DRow` | type | `lib/compiler/semcore.myc:791` | `type DRow = DRw(Vec[Pat], Binary{32})` | DRow: mirrors decision.rs::Row (a matrix row carrying its surface arm index). | Empirical/Declared |
| `compiler.semcore::DRow::DRw` | ctor | `lib/compiler/semcore.myc:791` | `DRw(Vec[Pat], Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::drow_pats` | fn | `lib/compiler/semcore.myc:793` | `fn drow_pats(r: DRow) => Vec[Pat]` | — | Empirical/Declared |
| `compiler.semcore::drow_arm` | fn | `lib/compiler/semcore.myc:796` | `fn drow_arm(r: DRow) => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::specialize_ctor_row` | fn | `lib/compiler/semcore.myc:801` | `fn specialize_ctor_row(rows: Vec[DRow], c: Bytes, a: Binary{32}) => Vec[DRow]` | specialize_ctor_row / specialize_lit_row: the DRow-carrying twin of the `_pat` versions above (FLAG-semcore-5 — no shared trait; the arm index rides through unchanged). | Empirical/Declared |
| `compiler.semcore::specialize_lit_row` | fn | `lib/compiler/semcore.myc:818` | `fn specialize_lit_row(rows: Vec[DRow], k: Bytes) => Vec[DRow]` | — | Empirical/Declared |
| `compiler.semcore::default_rows` | fn | `lib/compiler/semcore.myc:835` | `fn default_rows(rows: Vec[DRow]) => Vec[DRow]` | default_rows: `D(P)` over `DRow`s. | Empirical/Declared |
| `compiler.semcore::row_all_wild` | fn | `lib/compiler/semcore.myc:849` | `fn row_all_wild(pats: Vec[Pat]) => Bool` | row_all_wild: whether every column of one row is a wildcard. | Empirical/Declared |
| `compiler.semcore::first_nonwild_idx` | fn | `lib/compiler/semcore.myc:862` | `fn first_nonwild_idx(pats: Vec[Pat], idx: Binary{32}) => Binary{32}` | first_nonwild_idx: the FLAG-semcore-8 simplified column-selection heuristic — the first non-wildcard column of ONE row (called on the first row only, which the caller has already established is not all-wild). | Empirical/Declared |
| `compiler.semcore::extract_occ` | fn | `lib/compiler/semcore.myc:874` | `fn extract_occ(v: Vec[Vec[Binary{32}]], i: Binary{32}) => Pair[Vec[Binary{32}], Vec[Vec[Binary{32}]]]` | extract_occ / extract_ty / extract_pat: remove the `i`-th element, returning it paired with the remainder in original relative order (the building block of "rotate column `i` to the front"). | Empirical/Declared |
| `compiler.semcore::extract_ty` | fn | `lib/compiler/semcore.myc:883` | `fn extract_ty(v: Vec[Ty], i: Binary{32}) => Pair[Ty, Vec[Ty]]` | — | Empirical/Declared |
| `compiler.semcore::extract_pat` | fn | `lib/compiler/semcore.myc:892` | `fn extract_pat(v: Vec[Pat], i: Binary{32}) => Pair[Pat, Vec[Pat]]` | — | Empirical/Declared |
| `compiler.semcore::rotate_pat_to_front` | fn | `lib/compiler/semcore.myc:901` | `fn rotate_pat_to_front(v: Vec[Pat], i: Binary{32}) => Vec[Pat]` | — | Empirical/Declared |
| `compiler.semcore::rotate_rows_to_front` | fn | `lib/compiler/semcore.myc:904` | `fn rotate_rows_to_front(rows: Vec[DRow], i: Binary{32}) => Vec[DRow]` | — | Empirical/Declared |
| `compiler.semcore::u32_append` | fn | `lib/compiler/semcore.myc:912` | `fn u32_append(a: Vec[Binary{32}], b: Vec[Binary{32}]) => Vec[Binary{32}]` | u32_append / occ_append / cases_append: small list-splicing helpers. | Empirical/Declared |
| `compiler.semcore::occ_append` | fn | `lib/compiler/semcore.myc:915` | `fn occ_append(a: Vec[Vec[Binary{32}]], b: Vec[Vec[Binary{32}]]) => Vec[Vec[Binary{32}]]` | — | Empirical/Declared |
| `compiler.semcore::cases_append` | fn | `lib/compiler/semcore.myc:918` | `fn cases_append(a: Vec[Pair[Head, Tree]], b: Vec[Pair[Head, Tree]]) => Vec[Pair[Head, Tree]]` | — | Empirical/Declared |
| `compiler.semcore::child_occ` | fn | `lib/compiler/semcore.myc:922` | `fn child_occ(occ_rest: Vec[Vec[Binary{32}]], occ0: Vec[Binary{32}], a: Binary{32}) => Vec[Vec[Binary{32}]]` | child_occ: the `a` child occurrences `occ0 ++ [j]` (j = 0..a) followed by the remaining columns. | Empirical/Declared |
| `compiler.semcore::child_occ_acc` | fn | `lib/compiler/semcore.myc:926` | `fn child_occ_acc(occ0: Vec[Binary{32}], j: Binary{32}, a: Binary{32}) => Vec[Vec[Binary{32}]]` | — | Empirical/Declared |
| `compiler.semcore::gather_ctor_names` | fn | `lib/compiler/semcore.myc:933` | `fn gather_ctor_names(rows: Vec[DRow]) => Vec[Bytes]` | gather_ctor_names / gather_lit_names: first-seen, deduped head names in column 0. | Empirical/Declared |
| `compiler.semcore::gather_lit_names` | fn | `lib/compiler/semcore.myc:948` | `fn gather_lit_names(rows: Vec[DRow]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::signature_complete` | fn | `lib/compiler/semcore.myc:964` | `fn signature_complete(ty0: Ty, types: Vec[DataInfo], present: Vec[Bytes]) => Bool` | signature_complete: whether `present` covers `ty0`'s whole finite signature. | Empirical/Declared |
| `compiler.semcore::build_ctor_cases` | fn | `lib/compiler/semcore.myc:972` | `fn build_ctor_cases(depth: Binary{32}, types: Vec[DataInfo], rows: Vec[DRow], ty0: Ty, occ0: Vec[Binary{32}], occ_rest: Vec[Vec[Binary{32}]], tys_rest: Vec[Ty], present: Vec[Bytes]) => Result[Vec[Pair[Head, Tree]], Bytes]` | build_ctor_cases(_from): one `(Head::Ctor, subtree)` case per ctor of `ty0`'s signature that appears in `present`, in SIGNATURE order (mirrors decision.rs::compile_rows's ctor loop). | Empirical/Declared |
| `compiler.semcore::build_ctor_cases_from` | fn | `lib/compiler/semcore.myc:980` | `fn build_ctor_cases_from(depth: Binary{32}, types: Vec[DataInfo], rows: Vec[DRow], occ0: Vec[Binary{32}], occ_rest: Vec[Vec[Binary{32}]], tys_rest: Vec[Ty], ctors: Vec[CtorInfo], present: Vec[Bytes]) => Result[Vec[Pair[Head, Tree]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::build_lit_cases` | fn | `lib/compiler/semcore.myc:1004` | `fn build_lit_cases(depth: Binary{32}, types: Vec[DataInfo], rows: Vec[DRow], occ_rest: Vec[Vec[Binary{32}]], tys_rest: Vec[Ty], lits: Vec[Bytes]) => Result[Vec[Pair[Head, Tree]], Bytes]` | build_lit_cases: one `(Head::Lit, subtree)` case per literal head, first-seen order. | Empirical/Declared |
| `compiler.semcore::compile` | fn | `lib/compiler/semcore.myc:1019` | `fn compile(types: Vec[DataInfo], matrix: Vec[Vec[Pat]], arms: Vec[Binary{32}], occ: Vec[Vec[Binary{32}]], tys: Vec[Ty]) => Result[Tree, Bytes]` | compile: entry point — a fresh per-compilation budget (mirrors decision.rs::compile). | Empirical/Declared |
| `compiler.semcore::zip_rows` | fn | `lib/compiler/semcore.myc:1023` | `fn zip_rows(matrix: Vec[Vec[Pat]], arms: Vec[Binary{32}]) => Vec[DRow]` | — | Empirical/Declared |
| `compiler.semcore::compile_rows` | fn | `lib/compiler/semcore.myc:1033` | `fn compile_rows(depth: Binary{32}, types: Vec[DataInfo], rows: Vec[DRow], occ: Vec[Vec[Binary{32}]], tys: Vec[Ty]) => Result[Tree, Bytes]` | compile_rows: the budget-charged recursion (mirrors decision.rs::compile_rows). | Empirical/Declared |
| `compiler.semcore::compile_switch` | fn | `lib/compiler/semcore.myc:1046` | `fn compile_switch(depth: Binary{32}, types: Vec[DataInfo], rows: Vec[DRow], occ: Vec[Vec[Binary{32}]], tys: Vec[Ty], col: Binary{32}) => Result[Tree, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::has_reachable_fail` | fn | `lib/compiler/semcore.myc:1073` | `fn has_reachable_fail(t: Tree) => Bool` | has_reachable_fail: whether the tree contains a reachable `Fail` (mirrors decision.rs verbatim). | Empirical/Declared |
| `compiler.semcore::cases_any_reachable_fail` | fn | `lib/compiler/semcore.myc:1083` | `fn cases_any_reachable_fail(cases: Vec[Pair[Head, Tree]]) => Bool` | — | Empirical/Declared |
| `compiler.semcore::tree_eval` | fn | `lib/compiler/semcore.myc:1097` | `fn tree_eval(t: Tree, value: Pat) => Option[Binary{32}]` | tree_eval: a small TEST-ONLY reference evaluator (mirrors decision.rs's own module-doc-mentioned `eval_tree` reference — "it verifies the compiler; it does not run programs"). Walks the tree against one concrete (fully-ground, no `MpWild`) value pattern, returning the arm index reached, or `None` if a `Fail` is reached. Occurrence-indexed lookup into the concrete value is done via `pat_at_occ` (a small ground-value navigator — NOT part of decision.rs itself, purely this increment's differential-driving harness). | Empirical/Declared |
| `compiler.semcore::tree_eval_switch` | fn | `lib/compiler/semcore.myc:1104` | `fn tree_eval_switch(scrutinee: Pat, cases: Vec[Pair[Head, Tree]], deft: Option[Tree], value: Pat) => Option[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::tree_eval_deft` | fn | `lib/compiler/semcore.myc:1116` | `fn tree_eval_deft(deft: Option[Tree], value: Pat) => Option[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::head_matches` | fn | `lib/compiler/semcore.myc:1119` | `fn head_matches(h: Head, scrutinee: Pat) => Bool` | — | Empirical/Declared |
| `compiler.semcore::pat_at_occ` | fn | `lib/compiler/semcore.myc:1134` | `fn pat_at_occ(value: Pat, occ: Vec[Binary{32}]) => Pat` | pat_at_occ: navigate a ground value pattern via a field-index occurrence (root = `Nil`). | Empirical/Declared |
| `compiler.semcore::pat_nth` | fn | `lib/compiler/semcore.myc:1144` | `fn pat_nth(v: Vec[Pat], i: Binary{32}) => Pat` | — | Empirical/Declared |
| `compiler.semcore::Slot` | type | `lib/compiler/semcore.myc:1158` | `type Slot = Skip \| Live(Bytes) \| Moved(Bytes, Binary{32})` | Slot: mirrors affine.rs::Slot (one scope slot's affine state). | Empirical/Declared |
| `compiler.semcore::Slot::Live` | ctor | `lib/compiler/semcore.myc:1158` | `Live(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Slot::Moved` | ctor | `lib/compiler/semcore.myc:1158` | `Moved(Bytes, Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::Slot::Skip` | ctor | `lib/compiler/semcore.myc:1158` | `Skip` | — | Empirical/Declared |
| `compiler.semcore::UseOutcome` | type | `lib/compiler/semcore.myc:1161` | `type UseOutcome = NotAffine \| FirstUse \| DoubleUse(Bytes, Binary{32}, Binary{32})` | UseOutcome: mirrors affine.rs::UseOutcome (the outcome of recording a use at some index). | Empirical/Declared |
| `compiler.semcore::UseOutcome::DoubleUse` | ctor | `lib/compiler/semcore.myc:1161` | `DoubleUse(Bytes, Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::UseOutcome::FirstUse` | ctor | `lib/compiler/semcore.myc:1161` | `FirstUse` | — | Empirical/Declared |
| `compiler.semcore::UseOutcome::NotAffine` | ctor | `lib/compiler/semcore.myc:1161` | `NotAffine` | — | Empirical/Declared |
| `compiler.semcore::slot_for_ty` | fn | `lib/compiler/semcore.myc:1165` | `fn slot_for_ty(ty: Ty) => Slot` | slot_for_ty: the slot a fresh binding of type `ty` starts in (mirrors affine.rs::Slot::for_ty; the trailing-wildcard idiom — every non-`TySubstrate` type is `Skip`). | Empirical/Declared |
| `compiler.semcore::slots_seeded` | fn | `lib/compiler/semcore.myc:1170` | `fn slots_seeded(tys: Vec[Ty]) => Vec[Slot]` | slots_seeded: one slot per entry of `tys` (mirrors affine.rs::Tracker::seeded's per-parameter seeding, restated as a pure fn over the parameter TYPE list). | Empirical/Declared |
| `compiler.semcore::slots_use_at` | fn | `lib/compiler/semcore.myc:1176` | `fn slots_use_at(slots: Vec[Slot], idx: Binary{32}, ordinal: Binary{32}) => Pair[Vec[Slot], UseOutcome]` | slots_use_at: record a use (move) of the binding at scope index `idx`, returning the UPDATED slots vector paired with the outcome (mirrors affine.rs::Tracker::use_at; `ordinal` is the caller's own monotonic use counter — FLAG-semcore-7, no wrapper `UseSite` type). | Empirical/Declared |
| `compiler.semcore::slots_use_here` | fn | `lib/compiler/semcore.myc:1187` | `fn slots_use_here(slots: Vec[Slot], ordinal: Binary{32}) => Pair[Vec[Slot], UseOutcome]` | — | Empirical/Declared |
| `compiler.semcore::union_merge_into` | fn | `lib/compiler/semcore.myc:1202` | `fn union_merge_into(acc: Vec[Slot], other: Vec[Slot]) => Vec[Slot]` | merge_slot / union_merge_into: the conservative branch-merge rule (mirrors affine.rs verbatim — a slot moved in EITHER alternative is moved afterward). A length mismatch between `acc` and `other` is an internal-invariant violation (mirrors the Rust `debug_assert_eq!`); this total-fn port stops merging at the shorter side rather than panicking (never-silent by construction — no call site in this increment's differential produces mismatched snapshots). | Empirical/Declared |
| `compiler.semcore::merge_slot` | fn | `lib/compiler/semcore.myc:1211` | `fn merge_slot(a: Slot, o: Slot) => Slot` | — | Empirical/Declared |
| `compiler.semcore::CheckError` | type | `lib/compiler/semcore.myc:1225` | `type CheckError = CE(Bytes, Bytes)` | CheckError: mirrors checkty.rs::CheckError (site, message) — the one error surface `grade` needs; no checking LOGIC accompanies it (this file ports no checkty.rs logic at all). | Empirical/Declared |
| `compiler.semcore::CheckError::CE` | ctor | `lib/compiler/semcore.myc:1225` | `CE(Bytes, Bytes)` | — | Empirical/Declared |
| `compiler.semcore::ce_site` | fn | `lib/compiler/semcore.myc:1227` | `fn ce_site(e: CheckError) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::ce_message` | fn | `lib/compiler/semcore.myc:1230` | `fn ce_message(e: CheckError) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::fnenv_lookup` | fn | `lib/compiler/semcore.myc:1235` | `fn fnenv_lookup(fns: Vec[Pair[Bytes, FnDecl]], name: Bytes) => Option[FnDecl]` | FnEnv: the FLAG-semcore-4 ordered-assoc-list standing in for the checker's resolved `BTreeMap<String, FnDecl>` fn table. | Empirical/Declared |
| `compiler.semcore::ret_grade` | fn | `lib/compiler/semcore.myc:1245` | `fn ret_grade(fd: FnDecl) => Strength` | ret_grade / param_grade: the modular-bottom defaults (mirrors grade.rs verbatim — an unannotated demand/advertisement is `GDeclared`, the weakest grade). | Empirical/Declared |
| `compiler.semcore::param_grade` | fn | `lib/compiler/semcore.myc:1248` | `fn param_grade(p: Param) => Strength` | — | Empirical/Declared |
| `compiler.semcore::scope_lookup` | fn | `lib/compiler/semcore.myc:1253` | `fn scope_lookup(scope: Vec[Pair[Bytes, Strength]], name: Bytes) => Option[Strength]` | scope_lookup: a lexical stack of `(name, grade)`, most-recent-first (`Cons` = push; shadowing = head-first scan finds the most recent binding — mirrors the Rust `scope.iter().rev().find`). | Empirical/Declared |
| `compiler.semcore::require` | fn | `lib/compiler/semcore.myc:1263` | `fn require(have: Strength, demand: Strength, site: Bytes, what: Bytes) => Option[CheckError]` | require: the honesty check `have >= demand` (G-Sub; mirrors grade.rs::require, `Option`-returning rather than `Result<(),_>` — no unit type needed self-contained). | Empirical/Declared |
| `compiler.semcore::grade` | fn | `lib/compiler/semcore.myc:1270` | `fn grade(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, e: Expr) => Result[Strength, CheckError]` | grade: the budgeted entry point (mirrors grade.rs::Gx::grade's per-recursion budget charge). | Empirical/Declared |
| `compiler.semcore::grade_expr` | fn | `lib/compiler/semcore.myc:1279` | `fn grade_expr(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, e: Expr) => Result[Strength, CheckError]` | grade_expr: the per-variant dispatch (mirrors grade.rs::Gx::grade's big `match e { ... }`, field-for-field against ast.myc's `Expr` — every variant explicit, M-980 split-match idiom). | Empirical/Declared |
| `compiler.semcore::grade_path` | fn | `lib/compiler/semcore.myc:1308` | `fn grade_path(scope: Vec[Pair[Bytes, Strength]], p: Path) => Strength` | grade_path: G-Var — a single-segment path is a bound variable (`GExact` if unbound: a nullary constructor/constant); a multi-segment path is conservatively `GExact` too (the checker already refuses those; a residual one here is never reached by a well-formed body). | Empirical/Declared |
| `compiler.semcore::grade_lit` | fn | `lib/compiler/semcore.myc:1319` | `fn grade_lit(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, lit: Literal) => Result[Strength, CheckError]` | grade_lit: G-Const — a literal is `Exact` by construction; a list literal is the meet of its elements (G-Con). | Empirical/Declared |
| `compiler.semcore::grade_let` | fn | `lib/compiler/semcore.myc:1334` | `fn grade_let(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, name: Bytes, ty: Option[TypeRef], bound: Expr, body: Expr) => Result[Strength, CheckError]` | grade_let: G-Let / G-Weaken — grade the bound expr, weaken to the ascription if written, bind, then take the meet with the body's grade. | Empirical/Declared |
| `compiler.semcore::let_bind_grade` | fn | `lib/compiler/semcore.myc:1348` | `fn let_bind_grade(ty: Option[TypeRef], g_bound: Strength, site: Bytes, name: Bytes) => Result[Strength, CheckError]` | — | Empirical/Declared |
| `compiler.semcore::grade_if` | fn | `lib/compiler/semcore.myc:1363` | `fn grade_if(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, cond: Expr, conseq: Expr, alt: Expr) => Result[Strength, CheckError]` | grade_if: Design A — the condition is walked (to enforce demands inside it) but does NOT degrade the result; the result is the meet of both branch bodies. | Empirical/Declared |
| `compiler.semcore::bind_pattern` | fn | `lib/compiler/semcore.myc:1380` | `fn bind_pattern(scope: Vec[Pair[Bytes, Strength]], pat: Pattern, g_s: Strength) => Vec[Pair[Bytes, Strength]]` | bind_pattern(s): push every variable a pattern binds at grade `g_s` (G-Match/A's field-binder data-provenance rule); `POr` is unreachable post-check (the checker desugars it) — a defensive no-op here (this language has no abort/panic primitive to mirror Rust's invariant-violation panic, so the no-op IS the honest total-fn restatement, documented rather than silent). | Empirical/Declared |
| `compiler.semcore::bind_patterns` | fn | `lib/compiler/semcore.myc:1391` | `fn bind_patterns(scope: Vec[Pair[Bytes, Strength]], pats: Vec[Pattern], g_s: Strength) => Vec[Pair[Bytes, Strength]]` | — | Empirical/Declared |
| `compiler.semcore::grade_match` | fn | `lib/compiler/semcore.myc:1400` | `fn grade_match(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, scrutinee: Expr, arms: Vec[Arm]) => Result[Strength, CheckError]` | grade_match: G-Match/A — the scrutinee's grade does not appear in the result; the result is the meet of the arm bodies (each graded under its own pattern-bound scope). | Empirical/Declared |
| `compiler.semcore::grade_arms` | fn | `lib/compiler/semcore.myc:1407` | `fn grade_arms(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, arms: Vec[Arm], g_s: Strength) => Result[Strength, CheckError]` | — | Empirical/Declared |
| `compiler.semcore::grade_for` | fn | `lib/compiler/semcore.myc:1427` | `fn grade_for(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, x: Bytes, xs: Expr, acc: Bytes, init: Expr, body: Expr) => Result[Strength, CheckError]` | grade_for: the fixpoint-avoiding fold rule — the accumulator is graded at the BOTTOM (`GDeclared`) inside the body (sound without iterating to a fixpoint); the result is the meet of init/xs/body. | Empirical/Declared |
| `compiler.semcore::grade_colony` | fn | `lib/compiler/semcore.myc:1445` | `fn grade_colony(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, hyphae: Vec[Hypha]) => Result[Strength, CheckError]` | grade_colony: the colony's observable is its LAST hypha; leading hyphae are still walked (to enforce demands inside them) but do not contribute to the result grade. | Empirical/Declared |
| `compiler.semcore::grade_fuse` | fn | `lib/compiler/semcore.myc:1459` | `fn grade_fuse(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, left: Expr, right: Expr) => Result[Strength, CheckError]` | grade_fuse: DN-58 SSA/SSB — the meet of both operands (composition takes the weakest). | Empirical/Declared |
| `compiler.semcore::grade_reclaim` | fn | `lib/compiler/semcore.myc:1471` | `fn grade_reclaim(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, pol: Expr, body: Expr) => Result[Strength, CheckError]` | grade_reclaim: DN-58 SSB — the policy expr is walked (to surface any policy-grade violation), then the result is the body's grade. | Empirical/Declared |
| `compiler.semcore::grade_ascribe` | fn | `lib/compiler/semcore.myc:1480` | `fn grade_ascribe(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, inner: Expr, t: TypeRef) => Result[Strength, CheckError]` | grade_ascribe: G-Weaken — an `@ g` ascription demands the inferred grade satisfy `g`; the ascribed expr then carries `g`. A bare type ascription (no `@ g`) is grade-transparent. | Empirical/Declared |
| `compiler.semcore::meet_all` | fn | `lib/compiler/semcore.myc:1494` | `fn meet_all(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, es: Vec[Expr]) => Result[Strength, CheckError]` | meet_all: the meet of every expression's grade (`GExact` for an empty list — the meet identity). | Empirical/Declared |
| `compiler.semcore::app_head_fn` | fn | `lib/compiler/semcore.myc:1509` | `fn app_head_fn(fns: Vec[Pair[Bytes, FnDecl]], head: Expr) => Option[FnDecl]` | app_head_fn: resolve an `App` head to a KNOWN user function (a single-segment `Path` present in `fns`), else `None` (constructor / prim / trait-method — no graded signature in stage 1a). | Empirical/Declared |
| `compiler.semcore::grade_app` | fn | `lib/compiler/semcore.myc:1520` | `fn grade_app(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, head: Expr, args: Vec[Expr]) => Result[Strength, CheckError]` | grade_app: G-App (a known callee: check each arg against its param's demand, result is the callee's declared return grade) / G-Con-fallback (anything else: the conservative meet of args). | Empirical/Declared |
| `compiler.semcore::grade_call` | fn | `lib/compiler/semcore.myc:1527` | `fn grade_call(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, fd: FnDecl, args: Vec[Expr]) => Result[Strength, CheckError]` | — | Empirical/Declared |
| `compiler.semcore::check_args` | fn | `lib/compiler/semcore.myc:1534` | `fn check_args(depth: Binary{32}, fns: Vec[Pair[Bytes, FnDecl]], scope: Vec[Pair[Bytes, Strength]], site: Bytes, params: Vec[Param], args: Vec[Expr]) => Result[Binary{32}, CheckError]` | — | Empirical/Declared |
| `compiler.semcore::params_to_scope` | fn | `lib/compiler/semcore.myc:1552` | `fn params_to_scope(params: Vec[Param]) => Vec[Pair[Bytes, Strength]]` | params_to_scope / grade_fn_body: the thin per-body entry point this increment's differential drives directly (FLAG-semcore-9 — no whole-program `own_names`/`impl_methods` driver). | Empirical/Declared |
| `compiler.semcore::grade_fn_body` | fn | `lib/compiler/semcore.myc:1558` | `fn grade_fn_body(fns: Vec[Pair[Bytes, FnDecl]], fd: FnDecl) => Result[Strength, CheckError]` | — | Empirical/Declared |
| `compiler.semcore::eq_u` | fn | `lib/compiler/semcore.myc:1603` | `fn eq_u(a: Binary{32}, b: Binary{32}) => Bool` | — | Empirical/Declared |
| `compiler.semcore::beq` | fn | `lib/compiler/semcore.myc:1606` | `fn beq(a: Bytes, b: Bytes) => Bool` | — | Empirical/Declared |
| `compiler.semcore::and_` | fn | `lib/compiler/semcore.myc:1609` | `fn and_(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `compiler.semcore::width_is_var` | fn | `lib/compiler/semcore.myc:1614` | `fn width_is_var(w: Width) => Bool` | — | Empirical/Declared |
| `compiler.semcore::has_var` | fn | `lib/compiler/semcore.myc:1617` | `fn has_var(ty: Ty) => Bool` | — | Empirical/Declared |
| `compiler.semcore::any_has_var` | fn | `lib/compiler/semcore.myc:1632` | `fn any_has_var(tys: Vec[Ty]) => Bool` | — | Empirical/Declared |
| `compiler.semcore::type_head` | fn | `lib/compiler/semcore.myc:1642` | `fn type_head(ty: Ty) => Option[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::ty_subst_lookup` | fn | `lib/compiler/semcore.myc:1659` | `fn ty_subst_lookup(s: Vec[Pair[Bytes, Ty]], v: Bytes) => Option[Ty]` | — | Empirical/Declared |
| `compiler.semcore::subst_ty` | fn | `lib/compiler/semcore.myc:1667` | `fn subst_ty(ty: Ty, s: Vec[Pair[Bytes, Ty]]) => Ty` | — | Empirical/Declared |
| `compiler.semcore::subst_ty_list` | fn | `lib/compiler/semcore.myc:1682` | `fn subst_ty_list(tys: Vec[Ty], s: Vec[Pair[Bytes, Ty]]) => Vec[Ty]` | — | Empirical/Declared |
| `compiler.semcore::subst_binary_width` | fn | `lib/compiler/semcore.myc:1689` | `fn subst_binary_width(w: Width, s: Vec[Pair[Bytes, Ty]]) => Ty` | subst_binary_width / subst_ternary_width: DN-42/M-753 width-var substitution via the carrier convention — a width-var binding is stored as `v -> TyBinary(WdLit(n))` regardless of paradigm; on extraction we re-emit Binary or Ternary as appropriate. An unbound / unrecognised carrier is left as-is (defensive; mirrors the Rust `unwrap_or_else(\|\| ty.clone())` + `_ => ty.clone()`). | Empirical/Declared |
| `compiler.semcore::subst_ternary_width` | fn | `lib/compiler/semcore.myc:1701` | `fn subst_ternary_width(w: Width, s: Vec[Pair[Bytes, Ty]]) => Ty` | — | Empirical/Declared |
| `compiler.semcore::param_subst` | fn | `lib/compiler/semcore.myc:1715` | `fn param_subst(params: Vec[Bytes], args: Vec[Ty]) => Vec[Pair[Bytes, Ty]]` | — | Empirical/Declared |
| `compiler.semcore::scalar_eq` | fn | `lib/compiler/semcore.myc:1725` | `fn scalar_eq(a: Scalar, b: Scalar) => Bool` | — | Empirical/Declared |
| `compiler.semcore::sparsity_eq` | fn | `lib/compiler/semcore.myc:1733` | `fn sparsity_eq(a: Sparsity, b: Sparsity) => Bool` | — | Empirical/Declared |
| `compiler.semcore::width_eq` | fn | `lib/compiler/semcore.myc:1739` | `fn width_eq(a: Width, b: Width) => Bool` | — | Empirical/Declared |
| `compiler.semcore::ty_eq` | fn | `lib/compiler/semcore.myc:1745` | `fn ty_eq(a: Ty, b: Ty) => Bool` | — | Empirical/Declared |
| `compiler.semcore::ty_list_eq` | fn | `lib/compiler/semcore.myc:1760` | `fn ty_list_eq(a: Vec[Ty], b: Vec[Ty]) => Bool` | — | Empirical/Declared |
| `compiler.semcore::dec_digit` | fn | `lib/compiler/semcore.myc:1805` | `fn dec_digit(d: Binary{32}) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::ten32` | fn | `lib/compiler/semcore.myc:1817` | `fn ten32() => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::dec_u32` | fn | `lib/compiler/semcore.myc:1820` | `fn dec_u32(n: Binary{32}) => Bytes` | Base-10 render, big-endian: n < 10 -> one digit; else render n/10 then append n%10 (<= 10 iters). | Empirical/Declared |
| `compiler.semcore::tuple_type_name` | fn | `lib/compiler/semcore.myc:1829` | `fn tuple_type_name(n: Binary{32}) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::tuple_ctor_name` | fn | `lib/compiler/semcore.myc:1831` | `fn tuple_ctor_name(n: Binary{32}) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::tuple_params_from` | fn | `lib/compiler/semcore.myc:1834` | `fn tuple_params_from(i: Binary{32}, n: Binary{32}) => Vec[Bytes]` | tuple_params: the `n` type-parameter names `T0 .. T{n-1}` (Rust `(0..n).map(\|i\| format!("T{i}"))`). | Empirical/Declared |
| `compiler.semcore::vars_of` | fn | `lib/compiler/semcore.myc:1841` | `fn vars_of(params: Vec[Bytes]) => Vec[Ty]` | vars_of: one `TyVar(p)` field per parameter name (Rust `params.iter().map(\|p\| Ty::Var(p))`). | Empirical/Declared |
| `compiler.semcore::synthetic_tuple_data` | fn | `lib/compiler/semcore.myc:1844` | `fn synthetic_tuple_data(n: Binary{32}) => DataInfo` | — | Empirical/Declared |
| `compiler.semcore::typeref_base` | fn | `lib/compiler/semcore.myc:1850` | `fn typeref_base(t: TypeRef) => BaseType` | — | Empirical/Declared |
| `compiler.semcore::di_params` | fn | `lib/compiler/semcore.myc:1852` | `fn di_params(d: DataInfo) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::resolve_widthref_binary` | fn | `lib/compiler/semcore.myc:1857` | `fn resolve_widthref_binary(w: WidthRef) => Ty` | resolve_widthref_\*: a surface `WidthRef` (WLit/WName) becomes a checked `Width` under the given paradigm (Rust's per-`BaseType::Binary/Ternary` inline `match &t.base` arms). | Empirical/Declared |
| `compiler.semcore::resolve_widthref_ternary` | fn | `lib/compiler/semcore.myc:1860` | `fn resolve_widthref_ternary(w: WidthRef) => Ty` | — | Empirical/Declared |
| `compiler.semcore::vsa_kernel_model_id` | fn | `lib/compiler/semcore.myc:1864` | `fn vsa_kernel_model_id(surface: Bytes) => Bytes` | vsa_kernel_model_id: the surface->kernel VSA model-id canonicalization (checkty.rs; M-892). | Empirical/Declared |
| `compiler.semcore::tref_len` | fn | `lib/compiler/semcore.myc:1869` | `fn tref_len(v: Vec[TypeRef]) => Binary{32}` | tref_len / names_len: monomorphic length counters (Vec[TypeRef] / Vec[Bytes]) — the arity check. | Empirical/Declared |
| `compiler.semcore::names_len` | fn | `lib/compiler/semcore.myc:1872` | `fn names_len(v: Vec[Bytes]) => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::args_is_empty` | fn | `lib/compiler/semcore.myc:1875` | `fn args_is_empty(v: Vec[TypeRef]) => Bool` | — | Empirical/Declared |
| `compiler.semcore::resolve_base` | fn | `lib/compiler/semcore.myc:1878` | `fn resolve_base(types: Vec[DataInfo], tyvars: Vec[Bytes], b: BaseType) => Result[Ty, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::resolve_args` | fn | `lib/compiler/semcore.myc:1905` | `fn resolve_args(types: Vec[DataInfo], tyvars: Vec[Bytes], args: Vec[TypeRef]) => Result[Vec[Ty], Bytes]` | resolve_args: map resolve_ty over each surface arg, short-circuiting on the first Err (Rust's `for a in args { resolved.push(resolve_ty(..)?.0) }`; the guarantee of each arg is discarded). | Empirical/Declared |
| `compiler.semcore::resolve_named` | fn | `lib/compiler/semcore.myc:1917` | `fn resolve_named(types: Vec[DataInfo], tyvars: Vec[Bytes], name: Bytes, args: Vec[TypeRef]) => Result[Ty, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::resolve_tuple` | fn | `lib/compiler/semcore.myc:1932` | `fn resolve_tuple(types: Vec[DataInfo], tyvars: Vec[Bytes], elems: Vec[TypeRef]) => Result[Ty, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::resolve_ty` | fn | `lib/compiler/semcore.myc:1942` | `fn resolve_ty(types: Vec[DataInfo], tyvars: Vec[Bytes], t: TypeRef) => Result[Pair[Ty, Option[Strength]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::first_duplicate` | fn | `lib/compiler/semcore.myc:1953` | `fn first_duplicate(xs: Vec[Bytes]) => Option[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::first_duplicate_go` | fn | `lib/compiler/semcore.myc:1956` | `fn first_duplicate_go(seen: Vec[Bytes], xs: Vec[Bytes]) => Option[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::resolve_ctors` | fn | `lib/compiler/semcore.myc:1971` | `fn resolve_ctors(types: Vec[DataInfo], td: TypeDecl) => Result[Vec[CtorInfo], Bytes]` | resolve_ctors: for each surface `Ctor`, resolve every field `TypeRef` with the decl's type params in scope (reusing the `resolve_ty`/`resolve_args` family), refusing a duplicate constructor name. Mirrors checkty.rs::resolve_ctors arm-for-arm: the `seen` list carries the ctor-names-to-the-left for the dup check (Rust's `ctors.iter().any(..)`), and the result preserves declaration order (prepend-on-return). Never-silent `Err` on a duplicate ctor and on any field-resolution failure (G2/VR-5). The resolve_ty guarantee slot is discarded exactly as in Rust (`let (ty, _) = ..`). | Empirical/Declared |
| `compiler.semcore::resolve_ctors_go` | fn | `lib/compiler/semcore.myc:1974` | `fn resolve_ctors_go(types: Vec[DataInfo], tyvars: Vec[Bytes], seen: Vec[Bytes], cs: Vec[Ctor]) => Result[Vec[CtorInfo], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::TraitDecl` | type | `lib/compiler/semcore.myc:2005` | `type TraitDecl = TrD(Vis, Bytes, Vec[Bytes], Vec[FnSig])` | desugars them to `Type + Impl + Fn` items in a Phase-0 pass BEFORE registration (checkty.rs `check_phylum_inner`), so in production their tuples arrive via the desugared items. Porting the arms anyway keeps the port arm-for-arm faithful to `collect_tuple_arities` itself (never a silent gap — a raw-nodule caller gets exactly the oracle's answer). | Empirical/Declared |
| `compiler.semcore::TraitDecl::TrD` | ctor | `lib/compiler/semcore.myc:2005` | `TrD(Vis, Bytes, Vec[Bytes], Vec[FnSig])` | — | Empirical/Declared |
| `compiler.semcore::trd_sigs` | fn | `lib/compiler/semcore.myc:2007` | `fn trd_sigs(t: TraitDecl) => Vec[FnSig]` | — | Empirical/Declared |
| `compiler.semcore::ImplDecl` | type | `lib/compiler/semcore.myc:2010` | `type ImplDecl = ImD(Bytes, Vec[TypeRef], TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.semcore::ImplDecl::ImD` | ctor | `lib/compiler/semcore.myc:2010` | `ImD(Bytes, Vec[TypeRef], TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.semcore::imd_trait_args` | fn | `lib/compiler/semcore.myc:2012` | `fn imd_trait_args(i: ImplDecl) => Vec[TypeRef]` | — | Empirical/Declared |
| `compiler.semcore::imd_for_ty` | fn | `lib/compiler/semcore.myc:2015` | `fn imd_for_ty(i: ImplDecl) => TypeRef` | — | Empirical/Declared |
| `compiler.semcore::imd_methods` | fn | `lib/compiler/semcore.myc:2018` | `fn imd_methods(i: ImplDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.semcore::InherentImplDecl` | type | `lib/compiler/semcore.myc:2021` | `type InherentImplDecl = IID(TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.semcore::InherentImplDecl::IID` | ctor | `lib/compiler/semcore.myc:2021` | `IID(TypeRef, Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.semcore::iid_for_ty` | fn | `lib/compiler/semcore.myc:2023` | `fn iid_for_ty(i: InherentImplDecl) => TypeRef` | — | Empirical/Declared |
| `compiler.semcore::iid_methods` | fn | `lib/compiler/semcore.myc:2026` | `fn iid_methods(i: InherentImplDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.semcore::ViaDecl` | type | `lib/compiler/semcore.myc:2032` | `type ViaDecl = VD(Binary{32}, Bytes, Vec[TypeRef])` | ViaDecl: mirrors ast.rs::ViaDecl field-for-field (field_idx, trait_name, trait_args). No accessor — `collect_tuple_arities_item`'s Object arm does NOT walk `via_decls` (a dead field, kept only so `ObjectDecl` is field-for-field faithful; the oracle skips it too). | Empirical/Declared |
| `compiler.semcore::ViaDecl::VD` | ctor | `lib/compiler/semcore.myc:2032` | `VD(Binary{32}, Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.semcore::ObjectDecl` | type | `lib/compiler/semcore.myc:2034` | `type ObjectDecl = OD(Vis, Bytes, Vec[Bytes], Ctor, Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.semcore::ObjectDecl::OD` | ctor | `lib/compiler/semcore.myc:2034` | `OD(Vis, Bytes, Vec[Bytes], Ctor, Vec[ViaDecl], Vec[ImplDecl], Vec[FnDecl])` | — | Empirical/Declared |
| `compiler.semcore::od_ctor` | fn | `lib/compiler/semcore.myc:2036` | `fn od_ctor(o: ObjectDecl) => Ctor` | — | Empirical/Declared |
| `compiler.semcore::od_impls` | fn | `lib/compiler/semcore.myc:2039` | `fn od_impls(o: ObjectDecl) => Vec[ImplDecl]` | — | Empirical/Declared |
| `compiler.semcore::od_fns` | fn | `lib/compiler/semcore.myc:2042` | `fn od_fns(o: ObjectDecl) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.semcore::LowerRhs` | type | `lib/compiler/semcore.myc:2046` | `type LowerRhs = LrExpr(Expr) \| LrImpl(ImplDecl)` | LowerRhs / LowerDecl: mirror ast.rs::LowerRhs / ast.rs::LowerDecl (DN-54 §10). Only `rhs` grades. | Empirical/Declared |
| `compiler.semcore::LowerRhs::LrExpr` | ctor | `lib/compiler/semcore.myc:2046` | `LrExpr(Expr)` | — | Empirical/Declared |
| `compiler.semcore::LowerRhs::LrImpl` | ctor | `lib/compiler/semcore.myc:2046` | `LrImpl(ImplDecl)` | — | Empirical/Declared |
| `compiler.semcore::LowerDecl` | type | `lib/compiler/semcore.myc:2048` | `type LowerDecl = LD(Bytes, Vec[Bytes], LowerRhs)` | — | Empirical/Declared |
| `compiler.semcore::LowerDecl::LD` | ctor | `lib/compiler/semcore.myc:2048` | `LD(Bytes, Vec[Bytes], LowerRhs)` | — | Empirical/Declared |
| `compiler.semcore::ld_rhs` | fn | `lib/compiler/semcore.myc:2050` | `fn ld_rhs(l: LowerDecl) => LowerRhs` | — | Empirical/Declared |
| `compiler.semcore::UsePath` | type | `lib/compiler/semcore.myc:2056` | `type UsePath = UP(Path, Bool)` | UsePath: mirrors ast.rs::UsePath / ast.myc::UsePath field-for-field (`UP` ctor, same spelling as ast.myc — M-1013 STEP 4, resolve_imports increment). `path`: the imported path (a specific import names the item; a glob names the prefix whose `pub` names import). `glob`: true for `use a.b.\*`. | Empirical/Declared |
| `compiler.semcore::UsePath::UP` | ctor | `lib/compiler/semcore.myc:2056` | `UP(Path, Bool)` | — | Empirical/Declared |
| `compiler.semcore::usepath_path` | fn | `lib/compiler/semcore.myc:2058` | `fn usepath_path(u: UsePath) => Path` | — | Empirical/Declared |
| `compiler.semcore::usepath_glob` | fn | `lib/compiler/semcore.myc:2061` | `fn usepath_glob(u: UsePath) => Bool` | — | Empirical/Declared |
| `compiler.semcore::Item` | type | `lib/compiler/semcore.myc:2069` | `type Item = ItType(TypeDecl) \| ItFn(FnDecl) \| ItTrait(TraitDecl) \| ItImpl(ImplDecl) \| ItObject(ObjectDecl) \| ItLower(LowerDecl) \| ItInherentImpl(InherentImplDecl) \| ItUse(UsePath) \| ItOther` | Item gains ItUse (M-1013 STEP 4): the trimmed register-family mirror (FLAG-semcore-30's note) previously collapsed `Use`/`Default`/`Derive` into the single tuple-free `ItOther`, which was faithful for `collect_tuple_arities` (none of the three carry tuple-relevant content) but is NOT faithful for `resolve_imports`, which reads `Item::Use(UsePath)` directly (checkty.rs 1423-1533). `Default`/`Derive` remain folded into `ItOther` (still tuple-free and not read by resolve_imports). | Empirical/Declared |
| `compiler.semcore::Item::ItType` | ctor | `lib/compiler/semcore.myc:2070` | `ItType(TypeDecl)` | — | Empirical/Declared |
| `compiler.semcore::Item::ItFn` | ctor | `lib/compiler/semcore.myc:2071` | `ItFn(FnDecl)` | — | Empirical/Declared |
| `compiler.semcore::Item::ItTrait` | ctor | `lib/compiler/semcore.myc:2072` | `ItTrait(TraitDecl)` | — | Empirical/Declared |
| `compiler.semcore::Item::ItImpl` | ctor | `lib/compiler/semcore.myc:2073` | `ItImpl(ImplDecl)` | — | Empirical/Declared |
| `compiler.semcore::Item::ItObject` | ctor | `lib/compiler/semcore.myc:2074` | `ItObject(ObjectDecl)` | — | Empirical/Declared |
| `compiler.semcore::Item::ItLower` | ctor | `lib/compiler/semcore.myc:2075` | `ItLower(LowerDecl)` | — | Empirical/Declared |
| `compiler.semcore::Item::ItInherentImpl` | ctor | `lib/compiler/semcore.myc:2076` | `ItInherentImpl(InherentImplDecl)` | — | Empirical/Declared |
| `compiler.semcore::Item::ItUse` | ctor | `lib/compiler/semcore.myc:2077` | `ItUse(UsePath)` | — | Empirical/Declared |
| `compiler.semcore::Item::ItOther` | ctor | `lib/compiler/semcore.myc:2078` | `ItOther` | — | Empirical/Declared |
| `compiler.semcore::Nodule` | type | `lib/compiler/semcore.myc:2079` | `type Nodule = Nod(Path, Bool, Vec[Item])` | — | Empirical/Declared |
| `compiler.semcore::Nodule::Nod` | ctor | `lib/compiler/semcore.myc:2079` | `Nod(Path, Bool, Vec[Item])` | — | Empirical/Declared |
| `compiler.semcore::nodule_items` | fn | `lib/compiler/semcore.myc:2081` | `fn nodule_items(n: Nodule) => Vec[Item]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_typeref` | fn | `lib/compiler/semcore.myc:2099` | `fn collect_tuple_arities_typeref(t: TypeRef, acc: Vec[Binary{32}]) => Vec[Binary{32}]` | `_typeref` / `_trefs` / `_ctors` (below) are unchanged from PR-2 — already arm-for-arm faithful to checkty.rs::collect_tuple_arities_typeref and its list folds; the new `_sig` / `_pattern` / `_expr` / `_item` legs reuse them. | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_trefs` | fn | `lib/compiler/semcore.myc:2108` | `fn collect_tuple_arities_trefs(acc: Vec[Binary{32}], ts: Vec[TypeRef]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_ctors` | fn | `lib/compiler/semcore.myc:2114` | `fn collect_tuple_arities_ctors(acc: Vec[Binary{32}], cs: Vec[Ctor]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::expr_len` | fn | `lib/compiler/semcore.myc:2122` | `fn expr_len(v: Vec[Expr]) => Binary{32}` | expr_len / pattern_len: monomorphic length counters (Vec[Expr] / Vec[Pattern]) — the tuple-literal / tuple-pattern arity (mirrors Rust's `elems.len()` / `subs.len()`; the tref_len/pat_len twins). | Empirical/Declared |
| `compiler.semcore::pattern_len` | fn | `lib/compiler/semcore.myc:2125` | `fn pattern_len(v: Vec[Pattern]) => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_params` | fn | `lib/compiler/semcore.myc:2129` | `fn collect_tuple_arities_params(acc: Vec[Binary{32}], ps: Vec[Param]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_sig` | fn | `lib/compiler/semcore.myc:2135` | `fn collect_tuple_arities_sig(acc: Vec[Binary{32}], sig: FnSig) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_sigs` | fn | `lib/compiler/semcore.myc:2138` | `fn collect_tuple_arities_sigs(acc: Vec[Binary{32}], sigs: Vec[FnSig]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_pattern` | fn | `lib/compiler/semcore.myc:2147` | `fn collect_tuple_arities_pattern(acc: Vec[Binary{32}], p: Pattern) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_patterns` | fn | `lib/compiler/semcore.myc:2157` | `fn collect_tuple_arities_patterns(acc: Vec[Binary{32}], ps: Vec[Pattern]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_opt_typeref` | fn | `lib/compiler/semcore.myc:2164` | `fn collect_tuple_arities_opt_typeref(acc: Vec[Binary{32}], oty: Option[TypeRef]) => Vec[Binary{32}]` | collect_tuple_arities_opt_typeref: the `if let Some(t) = ty` guard on the `Let` arm's ascription. | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_expr` | fn | `lib/compiler/semcore.myc:2171` | `fn collect_tuple_arities_expr(acc: Vec[Binary{32}], e: Expr) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_exprs` | fn | `lib/compiler/semcore.myc:2202` | `fn collect_tuple_arities_exprs(acc: Vec[Binary{32}], es: Vec[Expr]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_arm` | fn | `lib/compiler/semcore.myc:2208` | `fn collect_tuple_arities_arm(acc: Vec[Binary{32}], a: Arm) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_arms` | fn | `lib/compiler/semcore.myc:2211` | `fn collect_tuple_arities_arms(acc: Vec[Binary{32}], arms: Vec[Arm]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_hyphae` | fn | `lib/compiler/semcore.myc:2217` | `fn collect_tuple_arities_hyphae(acc: Vec[Binary{32}], hs: Vec[Hypha]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_fndecl` | fn | `lib/compiler/semcore.myc:2225` | `fn collect_tuple_arities_fndecl(acc: Vec[Binary{32}], fd: FnDecl) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_fndecls` | fn | `lib/compiler/semcore.myc:2228` | `fn collect_tuple_arities_fndecls(acc: Vec[Binary{32}], fds: Vec[FnDecl]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_impl` | fn | `lib/compiler/semcore.myc:2234` | `fn collect_tuple_arities_impl(acc: Vec[Binary{32}], id: ImplDecl) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_impls` | fn | `lib/compiler/semcore.myc:2239` | `fn collect_tuple_arities_impls(acc: Vec[Binary{32}], ids: Vec[ImplDecl]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities_item` | fn | `lib/compiler/semcore.myc:2248` | `fn collect_tuple_arities_item(acc: Vec[Binary{32}], it: Item) => Vec[Binary{32}]` | collect_tuple_arities_item: checkty.rs::collect_tuple_arities_item, arm-for-arm. `ItOther` (Use / Default / Derive) is the oracle's `=> {}` no-op. Object/InherentImpl are pre-desugared in the real pipeline (see the mirror header above) but ported faithfully for a raw-nodule caller. | Empirical/Declared |
| `compiler.semcore::collect_tuple_arities` | fn | `lib/compiler/semcore.myc:2271` | `fn collect_tuple_arities(items: Vec[Item], acc: Vec[Binary{32}]) => Vec[Binary{32}]` | collect_tuple_arities: the FULL walk — every item's tuple-relevant content (checkty.rs §793). | Empirical/Declared |
| `compiler.semcore::register_tuple_arities` | fn | `lib/compiler/semcore.myc:2280` | `fn register_tuple_arities(types: Vec[DataInfo], arities: Vec[Binary{32}]) => Vec[DataInfo]` | register_tuple_arities: pre-register a synthetic `Tuple$N` for each collected arity, but only if absent (checkty.rs's `types.entry(tname).or_insert_with(\|\| synthetic_tuple_data(arity))`). A duplicate arity in `arities` is a no-op the second time (the presence check dedups it). | Empirical/Declared |
| `compiler.semcore::register_shells` | fn | `lib/compiler/semcore.myc:2296` | `fn register_shells(types: Vec[DataInfo], items: Vec[Item]) => Result[Vec[DataInfo], Bytes]` | register_shells: Pass 1 — a shell `DI(name, params, Nil)` per `Item::Type` so recursive/forward field references resolve (checkty.rs's first loop). Never-silent refusals, in Rust's exact order: a duplicate type PARAMETER (`first_duplicate`) is checked FIRST, then a duplicate type NAME (`types_lookup` over the accumulated registry — which includes the seeded prelude, so redeclaring `Bool` collides exactly as in Rust). EVERY non-Type item is skipped — the faithful mirror of the oracle's `if let Item::Type(td) = item` (only a type declaration declares a data type; the un-trimmed Fn/Trait/Impl/Object/Lower/InherentImpl/`ItOther` kinds are all non-declarations here). | Empirical/Declared |
| `compiler.semcore::types_set_ctors` | fn | `lib/compiler/semcore.myc:2316` | `fn types_set_ctors(types: Vec[DataInfo], name: Bytes, ctors: Vec[CtorInfo]) => Vec[DataInfo]` | types_set_ctors: replace the ctors of the (shell) entry named `name`, preserving its params — checkty.rs's `types.get_mut(&td.name).expect("registered above").ctors = ctors`. `register_shells` registered every `Item::Type`, so `name` is always present; a not-found (invariant-impossible) entry leaves the list unchanged (the wild-free, value-threaded analogue of Rust's `.expect`: no panic, no silent fabrication — G2). | Empirical/Declared |
| `compiler.semcore::fill_ctors` | fn | `lib/compiler/semcore.myc:2328` | `fn fill_ctors(types: Vec[DataInfo], items: Vec[Item]) => Result[Vec[DataInfo], Bytes]` | fill_ctors: Pass 2 — resolve each `Item::Type`'s ctors (all shells now present) and write them into the registry (checkty.rs's second loop). Never-silent: any field-resolution or duplicate-ctor failure short-circuits to `Err` (reusing PR-1's `resolve_ctors`). | Empirical/Declared |
| `compiler.semcore::register_types` | fn | `lib/compiler/semcore.myc:2344` | `fn register_types(types: Vec[DataInfo], nod: Nodule) => Result[Vec[DataInfo], Bytes]` | register_types: the two-pass data-declaration registration, value-threaded over the assoc-list registry (checkty.rs::register_types). Order matches Rust exactly: (0) the FULL M-826 tuple pre-pass (all legs — FLAG-semcore-30 CLOSED, PR-2b), (1) shell-register each type, (2) fill each type's ctors. Returns the mutated registry or the first never-silent refusal. | Empirical/Declared |
| `compiler.semcore::TraitInfo` | type | `lib/compiler/semcore.myc:2374` | `type TraitInfo = TrInfo(Bytes, Vec[Bytes], Vec[FnSig])` | TraitInfo: mirrors checkty.rs::TraitInfo (264-272) — { name, params (tyvar names), sigs (SURFACE FnSigs, stored verbatim) }. Modeled as a presence-checked `Vec[TraitInfo]` keyed by name, exactly as `register_types`' output is a `Vec[DataInfo]` (NOT a map). | Empirical/Declared |
| `compiler.semcore::TraitInfo::TrInfo` | ctor | `lib/compiler/semcore.myc:2374` | `TrInfo(Bytes, Vec[Bytes], Vec[FnSig])` | — | Empirical/Declared |
| `compiler.semcore::trinfo_name` | fn | `lib/compiler/semcore.myc:2376` | `fn trinfo_name(t: TraitInfo) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::trinfo_sigs` | fn | `lib/compiler/semcore.myc:2379` | `fn trinfo_sigs(t: TraitInfo) => Vec[FnSig]` | — | Empirical/Declared |
| `compiler.semcore::trd_name` | fn | `lib/compiler/semcore.myc:2384` | `fn trd_name(t: TraitDecl) => Bytes` | TraitDecl accessors: name (field 1) / params (field 2). `trd_sigs` (field 3) already exists above; `td_name`/`td_params` are TypeDecl's, so TraitDecl needs its own (distinct ADT). | Empirical/Declared |
| `compiler.semcore::trd_params` | fn | `lib/compiler/semcore.myc:2387` | `fn trd_params(t: TraitDecl) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::fnsig_type_params` | fn | `lib/compiler/semcore.myc:2392` | `fn fnsig_type_params(s: FnSig) => Vec[TypeParam]` | FnSig type-params accessor (FS field 1) — the 3 grade accessors above (name/value_params/ret) omitted it as unused; the trait pass needs it (method type-params → tyvar scope + bound checks). | Empirical/Declared |
| `compiler.semcore::tp_name` | fn | `lib/compiler/semcore.myc:2396` | `fn tp_name(t: TypeParam) => Bytes` | TypeParam / TraitRef accessors (checkty.rs reads `tp.name` / `tp.kind` / `tp.bounds` / `b.name`). | Empirical/Declared |
| `compiler.semcore::tp_kind` | fn | `lib/compiler/semcore.myc:2399` | `fn tp_kind(t: TypeParam) => ParamKind` | — | Empirical/Declared |
| `compiler.semcore::tp_bounds` | fn | `lib/compiler/semcore.myc:2402` | `fn tp_bounds(t: TypeParam) => Vec[TraitRef]` | — | Empirical/Declared |
| `compiler.semcore::traitref_name` | fn | `lib/compiler/semcore.myc:2405` | `fn traitref_name(r: TraitRef) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::fnsig_tyvar_names` | fn | `lib/compiler/semcore.myc:2412` | `fn fnsig_tyvar_names(s: FnSig) => Vec[Bytes]` | fnsig_tyvar_names: mirrors FnSig::param_names (ast.rs 447-460) — the method's TYPE-kind type-param names only (width params are dropped; they cannot stand in the tyvar scope, DN-42 §7). Distinct from the value-parameter `param_names(Vec[Param])` above (that maps `Vec[Param]`; this filters `Vec[TypeParam]` by `PkType`). | Empirical/Declared |
| `compiler.semcore::type_param_type_names` | fn | `lib/compiler/semcore.myc:2415` | `fn type_param_type_names(tps: Vec[TypeParam]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::traits_contains` | fn | `lib/compiler/semcore.myc:2425` | `fn traits_contains(traits: Vec[TraitInfo], name: Bytes) => Bool` | traits_contains: `traits.contains_key(name)` over the assoc-list registry (set membership by name). | Empirical/Declared |
| `compiler.semcore::check_sig_resolves` | fn | `lib/compiler/semcore.myc:2437` | `fn check_sig_resolves(types: Vec[DataInfo], tyvars: Vec[Bytes], sig: FnSig) => Result[Bool, Bytes]` | check_sig_resolves: checkty.rs 3088-3099 — every value-param type, then the ret, must `resolve_ty` under `tyvars`; `Ok(True)` (a unit sentinel; the resolved types + their guarantee slot are discarded exactly as Rust's `resolve_ty(...)?` ignores its value) or the FIRST `Err`. | Empirical/Declared |
| `compiler.semcore::check_params_resolve` | fn | `lib/compiler/semcore.myc:2446` | `fn check_params_resolve(types: Vec[DataInfo], tyvars: Vec[Bytes], params: Vec[Param]) => Result[Bool, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::register_traits` | fn | `lib/compiler/semcore.myc:2456` | `fn register_traits(types: Vec[DataInfo], nod: Nodule) => Result[Vec[TraitInfo], Bytes]` | register_traits: the two-pass entry (checkty.rs 3016-3083). Returns the registry or the first refusal. | Empirical/Declared |
| `compiler.semcore::register_traits_pass1` | fn | `lib/compiler/semcore.myc:2468` | `fn register_traits_pass1(types: Vec[DataInfo], acc: Vec[TraitInfo], items: Vec[Item]) => Result[Vec[TraitInfo], Bytes]` | PASS 1 — checkty.rs 3021-3057. Every non-Trait item is skipped (the mirror of `let Item::Trait(td) = item else { continue }`); each trait's checks run in the oracle's order (dup param -> dup name -> method checks -> insert). | Empirical/Declared |
| `compiler.semcore::check_trait_methods` | fn | `lib/compiler/semcore.myc:2488` | `fn check_trait_methods(types: Vec[DataInfo], trait_name: Bytes, params: Vec[Bytes], seen: Vec[Bytes], sigs: Vec[FnSig]) => Result[Bool, Bytes]` | check_trait_methods: the within-trait method loop (checkty.rs 3034-3048). `seen` tracks distinct method names; each sig resolves with `tyvars = trait.params ++ method.param_names()` (3045-3046). | Empirical/Declared |
| `compiler.semcore::register_traits_bounds` | fn | `lib/compiler/semcore.myc:2503` | `fn register_traits_bounds(all: Vec[TraitInfo], traits: Vec[TraitInfo]) => Result[Bool, Bytes]` | PASS 2 — checkty.rs 3063-3081. Over the COMPLETE registry (`all`), so a bound may forward-reference a later-declared trait. Every method type-parameter bound must name a KNOWN trait, else an explicit refusal naming the site + the offending bound (mirroring the oracle's message shape). | Empirical/Declared |
| `compiler.semcore::check_trait_sig_bounds` | fn | `lib/compiler/semcore.myc:2512` | `fn check_trait_sig_bounds(all: Vec[TraitInfo], trname: Bytes, sigs: Vec[FnSig]) => Result[Bool, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::check_sig_tp_bounds` | fn | `lib/compiler/semcore.myc:2521` | `fn check_sig_tp_bounds(all: Vec[TraitInfo], trname: Bytes, method: Bytes, tps: Vec[TypeParam]) => Result[Bool, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::unknown_bound_msg` | fn | `lib/compiler/semcore.myc:2533` | `fn unknown_bound_msg(trname: Bytes, method: Bytes, pname: Bytes, bound: Bytes) => Bytes` | unknown_bound_msg: the never-silent refusal text for pass 2 (checkty.rs 3068-3076) — names the site (`trait <T> method <m>`) and the offending bound (`<param>: <bound>`). Built in `let`-pieces to keep the concatenation legibly balanced. | Empirical/Declared |
| `compiler.semcore::check_tp_bounds` | fn | `lib/compiler/semcore.myc:2539` | `fn check_tp_bounds(all: Vec[TraitInfo], trname: Bytes, method: Bytes, pname: Bytes, bounds: Vec[TraitRef]) => Result[Bool, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::InstanceInfo` | type | `lib/compiler/semcore.myc:2578` | `type InstanceInfo = InstInfo(Bytes, Vec[Ty], Ty, Vec[Bytes])` | InstanceInfo: mirrors checkty.rs::InstanceInfo (277-286) -- { trait_name, trait_args (concrete Ty), for_ty (concrete Ty), methods (provided method names) }. Modeled as a presence-checked `Vec[InstanceInfo]` keyed by `(trait_name, type_head(for_ty))`, exactly as the other register-family outputs are Vecs (NOT maps). | Empirical/Declared |
| `compiler.semcore::InstanceInfo::InstInfo` | ctor | `lib/compiler/semcore.myc:2578` | `InstInfo(Bytes, Vec[Ty], Ty, Vec[Bytes])` | — | Empirical/Declared |
| `compiler.semcore::instinfo_trait_name` | fn | `lib/compiler/semcore.myc:2580` | `fn instinfo_trait_name(i: InstanceInfo) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::instinfo_for_ty` | fn | `lib/compiler/semcore.myc:2583` | `fn instinfo_for_ty(i: InstanceInfo) => Ty` | — | Empirical/Declared |
| `compiler.semcore::CoherenceView` | type | `lib/compiler/semcore.myc:2590` | `type CoherenceView = CV(Vec[Bytes], Vec[Bytes])` | CoherenceView: mirrors checkty.rs::CoherenceView (1318-1323) -- the pub-blind phylum-wide name view the orphan rule reads: (all trait names, all data-type names), each a `Vec[Bytes]` name-list (the FLAG-semcore-4 assoc-list stand-in for the oracle's two `BTreeSet<String>`; membership via `names_contains`). | Empirical/Declared |
| `compiler.semcore::CoherenceView::CV` | ctor | `lib/compiler/semcore.myc:2590` | `CV(Vec[Bytes], Vec[Bytes])` | — | Empirical/Declared |
| `compiler.semcore::cv_traits` | fn | `lib/compiler/semcore.myc:2592` | `fn cv_traits(c: CoherenceView) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::cv_types` | fn | `lib/compiler/semcore.myc:2595` | `fn cv_types(c: CoherenceView) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::imd_trait_name` | fn | `lib/compiler/semcore.myc:2600` | `fn imd_trait_name(i: ImplDecl) => Bytes` | ImplDecl trait-name accessor (ImD field 1) -- the register-family tuple pre-pass read only args/for_ty/methods; the impl pass needs the trait name (lookup + instance key + messages). | Empirical/Declared |
| `compiler.semcore::trinfo_params` | fn | `lib/compiler/semcore.myc:2604` | `fn trinfo_params(t: TraitInfo) => Vec[Bytes]` | TraitInfo params accessor (TrInfo field 2) -- the trait pass registered it; the arity check reads it. | Empirical/Declared |
| `compiler.semcore::traits_lookup` | fn | `lib/compiler/semcore.myc:2609` | `fn traits_lookup(traits: Vec[TraitInfo], name: Bytes) => Option[TraitInfo]` | traits_lookup: `traits.get(name)` over the assoc-list registry (the ENTRY, not just membership -- register_instances needs the `TraitInfo` for the arity + method-set checks). | Empirical/Declared |
| `compiler.semcore::impl_method_names` | fn | `lib/compiler/semcore.myc:2621` | `fn impl_method_names(id: ImplDecl) => Vec[Bytes]` | impl_method_names / trait_required_names: the impl's provided method NAMES (`m.sig.name`, i.e. `fnsig_name(fndecl_sig(m))`) and the trait's required method NAMES (`s.name` over the trait's sigs) -- the two name-sets `check_impl_method_set` compares (checkty.rs 3245-3248). | Empirical/Declared |
| `compiler.semcore::method_sig_names` | fn | `lib/compiler/semcore.myc:2624` | `fn method_sig_names(ms: Vec[FnDecl]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::trait_required_names` | fn | `lib/compiler/semcore.myc:2630` | `fn trait_required_names(sigs: Vec[FnSig]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::check_impl_method_set` | fn | `lib/compiler/semcore.myc:2641` | `fn check_impl_method_set(tr: TraitInfo, id: ImplDecl) => Result[Bool, Bytes]` | check_impl_method_set: the impl's method SET must EXACTLY match the trait's requirement SET (checkty.rs::check_impl_method_set 3243-3284). In the oracle's ORDER: (a) every required name is provided (else "missing method"); (b) every provided name is required (else "has method ... not in trait"); (c) no provided name appears twice (else "provides ... more than once"). All three are explicit never-silent refusals (G2) -- never a silently-filled or silently-dropped method. | Empirical/Declared |
| `compiler.semcore::check_methods_present` | fn | `lib/compiler/semcore.myc:2656` | `fn check_methods_present(trname: Bytes, required: Vec[Bytes], provided: Vec[Bytes]) => Result[Bool, Bytes]` | (a) every required method name must be present in `provided` (checkty.rs 3249-3256). | Empirical/Declared |
| `compiler.semcore::check_methods_no_extra` | fn | `lib/compiler/semcore.myc:2666` | `fn check_methods_no_extra(trname: Bytes, required: Vec[Bytes], provided: Vec[Bytes]) => Result[Bool, Bytes]` | (b) every provided method name must be required -- no extra methods (checkty.rs 3257-3267). | Empirical/Declared |
| `compiler.semcore::type_local` | fn | `lib/compiler/semcore.myc:2680` | `fn type_local(coherence_types: Vec[Bytes], for_ty: Ty) => Bool` | type_local: the orphan rule's `type_local` arm (checkty.rs 3182-3198), matching the `for_ty` variant EXACTLY as the oracle -- a `Data` head is local iff its name is in coherence.types; every primitive repr (Binary/Ternary/Dense/Vsa/Substrate/Seq/Bytes/Float) is phylum-owned (always true); `Var`/`Fn` are not legal heads (false -- kept for exhaustiveness, never a silent accept, G2; `type_head` refuses them upstream so these two arms are unreachable in practice). | Empirical/Declared |
| `compiler.semcore::orphan_ok` | fn | `lib/compiler/semcore.myc:2696` | `fn orphan_ok(coherence: CoherenceView, trait_name: Bytes, for_ty: Ty) => Bool` | orphan_ok: legal iff trait_local OR type_local (checkty.rs 3181-3199). | Empirical/Declared |
| `compiler.semcore::instance_head_eq` | fn | `lib/compiler/semcore.myc:2705` | `fn instance_head_eq(for_ty: Ty, head: Bytes) => Bool` | instance_head_eq: does `for_ty`'s head equal `head`? A stored instance always has a `Some` head (it passed the type_head check), so the `None` arm is an unreachable-in-practice `False` -- never a silent match (G2). | Empirical/Declared |
| `compiler.semcore::instances_contains_key` | fn | `lib/compiler/semcore.myc:2714` | `fn instances_contains_key(acc: Vec[InstanceInfo], trait_name: Bytes, head: Bytes) => Bool` | instances_contains_key: `instances.contains_key((trait_name, head))` over the assoc-list registry -- the global-uniqueness scan (checkty.rs 3213-3214). The stored `(trait, head)` is recomputed from the instance's `for_ty` (the registry keeps the full for_ty, not the erased head). | Empirical/Declared |
| `compiler.semcore::register_instances` | fn | `lib/compiler/semcore.myc:2726` | `fn register_instances(types: Vec[DataInfo], traits: Vec[TraitInfo], coherence: CoherenceView, nod: Nodule) => Result[Vec[InstanceInfo], Bytes]` | register_instances: the two-level loop (checkty.rs 3130-3237). Returns the instance registry or the FIRST refusal. `register_one_instance` threads the accumulated registry so the uniqueness scan sees earlier-registered instances (the oracle's growing `BTreeMap`). | Empirical/Declared |
| `compiler.semcore::register_instances_go` | fn | `lib/compiler/semcore.myc:2729` | `fn register_instances_go(types: Vec[DataInfo], traits: Vec[TraitInfo], coherence: CoherenceView, acc: Vec[InstanceInfo], items: Vec[Item]) => Result[Vec[InstanceInfo], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::register_one_instance` | fn | `lib/compiler/semcore.myc:2743` | `fn register_one_instance(types: Vec[DataInfo], traits: Vec[TraitInfo], coherence: CoherenceView, acc: Vec[InstanceInfo], id: ImplDecl) => Result[Vec[InstanceInfo], Bytes]` | register_one_instance: the per-impl body (checkty.rs 3132-3235) -- the eight checks in the oracle's EXACT order. `resolve_ty`'s guarantee slot is discarded (`let (for_ty, _) = ..`), matching the oracle. | Empirical/Declared |
| `compiler.semcore::arity_mismatch_msg` | fn | `lib/compiler/semcore.myc:2774` | `fn arity_mismatch_msg(trname: Bytes, want: Binary{32}, got: Binary{32}) => Bytes` | The three never-silent refusal messages, mirroring the oracle's site + reason wording (checkty.rs 3152-3163 / 3199-3209 / 3214-3224). Built in `bytes_concat` pieces (the file's established idiom). | Empirical/Declared |
| `compiler.semcore::orphan_msg` | fn | `lib/compiler/semcore.myc:2777` | `fn orphan_msg(trname: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::overlapping_msg` | fn | `lib/compiler/semcore.myc:2780` | `fn overlapping_msg(trname: Bytes, head: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::same_ty_len` | fn | `lib/compiler/semcore.myc:2785` | `fn same_ty_len(a: Vec[Ty], b: Vec[Ty]) => Bool` | — | Empirical/Declared |
| `compiler.semcore::unify_eq_or_mismatch` | fn | `lib/compiler/semcore.myc:2793` | `fn unify_eq_or_mismatch(decl: Ty, actual: Ty, s: Vec[Pair[Bytes, Ty]]) => Result[Vec[Pair[Bytes, Ty]], Bytes]` | The `_ if decl == actual` fallback (arm 8) + the `_ => Err` mismatch (arm 9), and the collapsed FLAG-semcore-14 dead arm: an unhandled pair unifies iff it is already structurally equal. | Empirical/Declared |
| `compiler.semcore::unify_var` | fn | `lib/compiler/semcore.myc:2801` | `fn unify_var(v: Bytes, actual: Ty, s: Vec[Pair[Bytes, Ty]]) => Result[Vec[Pair[Bytes, Ty]], Bytes]` | unify_var: the `(Ty::Var(v), _)` arm — a prior binding to a DIFFERENT type is a never-silent conflict; otherwise bind `v -> actual` (VR-5/G2). | Empirical/Declared |
| `compiler.semcore::unify_width_carrier` | fn | `lib/compiler/semcore.myc:2813` | `fn unify_width_carrier(v: Bytes, carrier: Ty, s: Vec[Pair[Bytes, Ty]]) => Result[Vec[Pair[Bytes, Ty]], Bytes]` | unify_width_carrier: the shared width-var binding logic. The carrier is ALWAYS a `TyBinary` (`WdLit(n)` for a concrete width, `WdVar(v2)` for a width-var) regardless of the paradigm — the DN-42/M-753 carrier convention `subst_ty` reads back. A conflicting prior binding is never-silent. | Empirical/Declared |
| `compiler.semcore::unify_binary_width` | fn | `lib/compiler/semcore.myc:2825` | `fn unify_binary_width(v: Bytes, actual: Ty, decl: Ty, s: Vec[Pair[Bytes, Ty]]) => Result[Vec[Pair[Bytes, Ty]], Bytes]` | unify_width: a width-var `decl` (Binary or Ternary) against an actual of the SAME paradigm. A concrete-width actual binds `v -> TyBinary(WdLit(n))`; a width-var actual binds `v -> TyBinary(WdVar(v2))`. Any cross-paradigm / non-width actual falls through to eq/mismatch. | Empirical/Declared |
| `compiler.semcore::unify_ternary_width` | fn | `lib/compiler/semcore.myc:2834` | `fn unify_ternary_width(v: Bytes, actual: Ty, decl: Ty, s: Vec[Pair[Bytes, Ty]]) => Result[Vec[Pair[Bytes, Ty]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::unify` | fn | `lib/compiler/semcore.myc:2843` | `fn unify(decl: Ty, actual: Ty, s: Vec[Pair[Bytes, Ty]]) => Result[Vec[Pair[Bytes, Ty]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::unify_list` | fn | `lib/compiler/semcore.myc:2876` | `fn unify_list(a1: Vec[Ty], a2: Vec[Ty], s: Vec[Pair[Bytes, Ty]]) => Result[Vec[Pair[Bytes, Ty]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::three32` | fn | `lib/compiler/semcore.myc:2919` | `fn three32() => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::replace_dash_at` | fn | `lib/compiler/semcore.myc:2924` | `fn replace_dash_at(s: Bytes, i: Binary{32}, n: Binary{32}) => Bytes` | replace_dash: `model.replace('-', '_')` (the VSA kernel-model-id fragment is `$`-joined and its `-` — not an identifier char — maps to `_`; injective over the model-id alphabet). O(len) over the short model id via 1-char slices. | Empirical/Declared |
| `compiler.semcore::replace_dash` | fn | `lib/compiler/semcore.myc:2934` | `fn replace_dash(s: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::strip_fn_prefix` | fn | `lib/compiler/semcore.myc:2937` | `fn strip_fn_prefix(a: Bytes) => Bytes` | strip_fn_prefix: `arrow.strip_prefix("Fn$").unwrap_or(arrow)` — strip a leading "Fn$" if present. | Empirical/Declared |
| `compiler.semcore::scalar_tag` | fn | `lib/compiler/semcore.myc:2947` | `fn scalar_tag(s: Scalar) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::vsa_sp_tag` | fn | `lib/compiler/semcore.myc:2951` | `fn vsa_sp_tag(sp: Sparsity) => Bytes` | vsa_sp_tag: the VSA sparsity fragment (`Dense` -> "Dn", `Sparse(k)` -> "Sp{k}"). | Empirical/Declared |
| `compiler.semcore::mangle_ty_args` | fn | `lib/compiler/semcore.myc:2956` | `fn mangle_ty_args(args: Vec[Ty]) => Bytes` | mangle_ty_args: the `$`-joined argument fragments of an applied data type / a decl's type args (`for a in args { push('$'); push(mangle_ty(a)) }`). | Empirical/Declared |
| `compiler.semcore::mangle_ty` | fn | `lib/compiler/semcore.myc:2963` | `fn mangle_ty(t: Ty) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::mangle_decl` | fn | `lib/compiler/semcore.myc:2983` | `fn mangle_decl(name: Bytes, targs: Vec[Ty]) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::mangle_ctor` | fn | `lib/compiler/semcore.myc:2986` | `fn mangle_ctor(name: Bytes, targs: Vec[Ty]) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::mangle_method` | fn | `lib/compiler/semcore.myc:2989` | `fn mangle_method(method: Bytes, trait_name: Bytes, for_ty: Ty) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::mangle_ty_or_fn` | fn | `lib/compiler/semcore.myc:2995` | `fn mangle_ty_or_fn(t: Ty) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::mangle_arrow` | fn | `lib/compiler/semcore.myc:2998` | `fn mangle_arrow(a: Ty, b: Ty) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::apply_fn_name` | fn | `lib/compiler/semcore.myc:3002` | `fn apply_fn_name(arrow: Bytes) => Bytes` | apply_fn_name: the dispatcher name for an arrow mangle `Fn$A$B` -> `apply$A$B`. | Empirical/Declared |
| `compiler.semcore::warg_tag` | fn | `lib/compiler/semcore.myc:3007` | `fn warg_tag(w: Width) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::warg_joints` | fn | `lib/compiler/semcore.myc:3010` | `fn warg_joints(wargs: Vec[Width]) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::mangle_decl_with_wargs` | fn | `lib/compiler/semcore.myc:3013` | `fn mangle_decl_with_wargs(name: Bytes, targs: Vec[Ty], wargs: Vec[Width]) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::pairs_is_empty` | fn | `lib/compiler/semcore.myc:3017` | `fn pairs_is_empty(p: Vec[Pair[Binary{32}, Bytes]]) => Bool` | pairs_is_empty: the `fn_args.is_empty() && dyn_fns.is_empty()` guard (over Vec[Pair[u32, Bytes]]). | Empirical/Declared |
| `compiler.semcore::hof_fn_joints` | fn | `lib/compiler/semcore.myc:3021` | `fn hof_fn_joints(fa: Vec[Pair[Binary{32}, Bytes]]) => Bytes` | hof_fn_joints: the static fn-argument segments `%{idx}:{callee}` (the `%` fresh-var joint). | Empirical/Declared |
| `compiler.semcore::hof_dyn_joints` | fn | `lib/compiler/semcore.myc:3029` | `fn hof_dyn_joints(df: Vec[Pair[Binary{32}, Bytes]]) => Bytes` | hof_dyn_joints: the dynamic (kept-closure) fn-argument segments `~{idx}:{arrow}` (the `~` joint). | Empirical/Declared |
| `compiler.semcore::mangle_hof_decl` | fn | `lib/compiler/semcore.myc:3036` | `fn mangle_hof_decl(name: Bytes, targs: Vec[Ty], wargs: Vec[Width], fa: Vec[Pair[Binary{32}, Bytes]], df: Vec[Pair[Binary{32}, Bytes]]) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::names_snoc` | fn | `lib/compiler/semcore.myc:3077` | `fn names_snoc(xs: Vec[Bytes], n: Bytes) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::names_append` | fn | `lib/compiler/semcore.myc:3081` | `fn names_append(a: Vec[Bytes], b: Vec[Bytes]) => Vec[Bytes]` | names_append: concatenate two name lists (a pattern's binder-set union across sub-patterns). | Empirical/Declared |
| `compiler.semcore::ensure_bound` | fn | `lib/compiler/semcore.myc:3085` | `fn ensure_bound(bound: Vec[Bytes], n: Bytes) => Vec[Bytes]` | ensure_bound: `bound.insert(n)` as a set op — add iff absent (an already-bound name is unchanged). | Empirical/Declared |
| `compiler.semcore::union_names` | fn | `lib/compiler/semcore.myc:3089` | `fn union_names(bound: Vec[Bytes], names: Vec[Bytes]) => Vec[Bytes]` | union_names: extend `bound` by every name in `names` (the arm/lambda/for scope extension). | Empirical/Declared |
| `compiler.semcore::param_names` | fn | `lib/compiler/semcore.myc:3093` | `fn param_names(params: Vec[Param]) => Vec[Bytes]` | param_names: the value-parameter names of a lambda (Rust `params.iter().map(\|p\| p.name)`). | Empirical/Declared |
| `compiler.semcore::pattern_binder_names` | fn | `lib/compiler/semcore.myc:3097` | `fn pattern_binder_names(pat: Pattern, depth: Binary{32}) => Result[Vec[Bytes], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::pattern_binder_names_list` | fn | `lib/compiler/semcore.myc:3110` | `fn pattern_binder_names_list(subs: Vec[Pattern], depth: Binary{32}) => Result[Vec[Bytes], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::fvw` | fn | `lib/compiler/semcore.myc:3124` | `fn fvw(e: Expr, bound: Vec[Bytes], seen: Vec[Bytes], out: Vec[Bytes], depth: Binary{32}) => Result[Pair[Vec[Bytes], Vec[Bytes]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::fvw_list` | fn | `lib/compiler/semcore.myc:3192` | `fn fvw_list(es: Vec[Expr], bound: Vec[Bytes], seen: Vec[Bytes], out: Vec[Bytes], depth: Binary{32}) => Result[Pair[Vec[Bytes], Vec[Bytes]], Bytes]` | fvw_list: walk a Vec[Expr] left-to-right, threading (seen, out) (Lit(List)/App args/TupleLit). | Empirical/Declared |
| `compiler.semcore::fvw_hyphae` | fn | `lib/compiler/semcore.myc:3202` | `fn fvw_hyphae(hs: Vec[Hypha], bound: Vec[Bytes], seen: Vec[Bytes], out: Vec[Bytes], depth: Binary{32}) => Result[Pair[Vec[Bytes], Vec[Bytes]], Bytes]` | fvw_hyphae: walk each hypha's body (Colony) — forage clauses are not free-var positions here. | Empirical/Declared |
| `compiler.semcore::fvw_arms` | fn | `lib/compiler/semcore.myc:3213` | `fn fvw_arms(arms: Vec[Arm], bound: Vec[Bytes], seen: Vec[Bytes], out: Vec[Bytes], depth: Binary{32}) => Result[Pair[Vec[Bytes], Vec[Bytes]], Bytes]` | fvw_arms: per arm, extend `bound` by the arm pattern's binder set, walk the arm body; `bound` resets to the match scope between arms (the Rust insert/remove pair, collapsed). | Empirical/Declared |
| `compiler.semcore::free_vars` | fn | `lib/compiler/semcore.myc:3226` | `fn free_vars(e: Expr) => Result[Vec[Bytes], Bytes]` | free_vars: the pub entry — an empty scope/seen/out, returning the ordered capture list. | Empirical/Declared |
| `compiler.semcore::Bind` | type | `lib/compiler/semcore.myc:3270` | `type Bind = Bnd(Bytes, Ty, Vec[Binary{32}])` | — | Empirical/Declared |
| `compiler.semcore::Bind::Bnd` | ctor | `lib/compiler/semcore.myc:3270` | `Bnd(Bytes, Ty, Vec[Binary{32}])` | — | Empirical/Declared |
| `compiler.semcore::snoc_bind` | fn | `lib/compiler/semcore.myc:3272` | `fn snoc_bind(xs: Vec[Bind], b: Bind) => Vec[Bind]` | — | Empirical/Declared |
| `compiler.semcore::is_bindigit` | fn | `lib/compiler/semcore.myc:3276` | `fn is_bindigit(ch: Bytes) => Bool` | — | Empirical/Declared |
| `compiler.semcore::count_bindigits_at` | fn | `lib/compiler/semcore.myc:3280` | `fn count_bindigits_at(s: Bytes, i: Binary{32}, n: Binary{32}) => Binary{32}` | count_bindigits: `s.chars().filter(\|c\| \*c == '0' \|\| \*c == '1').count()` (the binary literal width). | Empirical/Declared |
| `compiler.semcore::count_bindigits` | fn | `lib/compiler/semcore.myc:3287` | `fn count_bindigits(s: Bytes) => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::filter_bindigits_at` | fn | `lib/compiler/semcore.myc:3290` | `fn filter_bindigits_at(s: Bytes, i: Binary{32}, n: Binary{32}) => Bytes` | filter_bindigits: the 0/1 chars of `s` concatenated (the `literal_key` `b:` body). | Empirical/Declared |
| `compiler.semcore::filter_bindigits` | fn | `lib/compiler/semcore.myc:3297` | `fn filter_bindigits(s: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::filter_no_us_at` | fn | `lib/compiler/semcore.myc:3300` | `fn filter_no_us_at(s: Bytes, i: Binary{32}, n: Binary{32}) => Bytes` | filter_no_underscore: `s.chars().filter(\|c\| \*c != '_')` (the `literal_key` `by:` body). | Empirical/Declared |
| `compiler.semcore::filter_no_underscore` | fn | `lib/compiler/semcore.myc:3307` | `fn filter_no_underscore(s: Bytes) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::lit_ty_of` | fn | `lib/compiler/semcore.myc:3310` | `fn lit_ty_of(l: Literal) => Result[Ty, Bytes]` | — | Empirical/Declared |
| `compiler.semcore::literal_key` | fn | `lib/compiler/semcore.myc:3324` | `fn literal_key(l: Literal) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::ctor_lookup` | fn | `lib/compiler/semcore.myc:3338` | `fn ctor_lookup(ctors: Vec[CtorInfo], name: Bytes) => Option[CtorInfo]` | — | Empirical/Declared |
| `compiler.semcore::pat_count` | fn | `lib/compiler/semcore.myc:3345` | `fn pat_count(v: Vec[Pattern]) => Binary{32}` | pat_count: the arity of a sub-pattern list (Vec[Pattern] length). | Empirical/Declared |
| `compiler.semcore::is_float_lit` | fn | `lib/compiler/semcore.myc:3348` | `fn is_float_lit(l: Literal) => Bool` | — | Empirical/Declared |
| `compiler.semcore::normalize_lit` | fn | `lib/compiler/semcore.myc:3350` | `fn normalize_lit(lit: Literal, expected: Ty, binds: Vec[Bind]) => Result[Pair[Pat, Vec[Bind]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::normalize_ident` | fn | `lib/compiler/semcore.myc:3362` | `fn normalize_ident(types: Vec[DataInfo], n: Bytes, expected: Ty, occ: Vec[Binary{32}], binds: Vec[Bind]) => Result[Pair[Pat, Vec[Bind]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::normalize_fields` | fn | `lib/compiler/semcore.myc:3377` | `fn normalize_fields(types: Vec[DataInfo], subs: Vec[Pattern], fields: Vec[Ty], s: Vec[Pair[Bytes, Ty]], occ: Vec[Binary{32}], i: Binary{32}, binds: Vec[Bind]) => Result[Pair[Vec[Pat], Vec[Bind]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::normalize_ctor` | fn | `lib/compiler/semcore.myc:3397` | `fn normalize_ctor(types: Vec[DataInfo], n: Bytes, subs: Vec[Pattern], expected: Ty, occ: Vec[Binary{32}], binds: Vec[Bind]) => Result[Pair[Pat, Vec[Bind]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::normalize_pattern` | fn | `lib/compiler/semcore.myc:3416` | `fn normalize_pattern(types: Vec[DataInfo], pat: Pattern, expected: Ty, occ: Vec[Binary{32}], binds: Vec[Bind]) => Result[Pair[Pat, Vec[Bind]], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::ScalarK` | type | `lib/compiler/semcore.myc:3531` | `type ScalarK = SkF16 \| SkBf16 \| SkF32 \| SkF64` | — | Empirical/Declared |
| `compiler.semcore::ScalarK::SkBf16` | ctor | `lib/compiler/semcore.myc:3531` | `SkBf16` | — | Empirical/Declared |
| `compiler.semcore::ScalarK::SkF16` | ctor | `lib/compiler/semcore.myc:3531` | `SkF16` | — | Empirical/Declared |
| `compiler.semcore::ScalarK::SkF32` | ctor | `lib/compiler/semcore.myc:3531` | `SkF32` | — | Empirical/Declared |
| `compiler.semcore::ScalarK::SkF64` | ctor | `lib/compiler/semcore.myc:3531` | `SkF64` | — | Empirical/Declared |
| `compiler.semcore::SparsityC` | type | `lib/compiler/semcore.myc:3534` | `type SparsityC = ScDense \| ScSparse(Binary{32})` | SparsityC: mirrors mycelium_core::SparsityClass (repr.rs). Sparse carries the max_active passthrough. | Empirical/Declared |
| `compiler.semcore::SparsityC::ScDense` | ctor | `lib/compiler/semcore.myc:3534` | `ScDense` | — | Empirical/Declared |
| `compiler.semcore::SparsityC::ScSparse` | ctor | `lib/compiler/semcore.myc:3534` | `ScSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::FloatW` | type | `lib/compiler/semcore.myc:3537` | `type FloatW = FwF64` | FloatW: mirrors mycelium_core::FloatWidth (repr.rs) — F64-only at introduction (ADR-040 FLAG-1). | Empirical/Declared |
| `compiler.semcore::FloatW::FwF64` | ctor | `lib/compiler/semcore.myc:3537` | `FwF64` | — | Empirical/Declared |
| `compiler.semcore::Repr` | type | `lib/compiler/semcore.myc:3540` | `type Repr = RBinary(Binary{32}) \| RTernary(Binary{32}) \| RDense(Binary{32}, ScalarK) \| RVsa(Bytes, Binary{32}, SparsityC) \| RSeq(Repr, Binary{32}) \| RFloat(FloatW) \| RBytes` | Repr: mirrors mycelium_core::Repr (repr.rs), the four closed paradigm kinds + Seq/Float/Bytes. | Empirical/Declared |
| `compiler.semcore::Repr::RBinary` | ctor | `lib/compiler/semcore.myc:3541` | `RBinary(Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::Repr::RTernary` | ctor | `lib/compiler/semcore.myc:3542` | `RTernary(Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::Repr::RDense` | ctor | `lib/compiler/semcore.myc:3543` | `RDense(Binary{32}, ScalarK)` | — | Empirical/Declared |
| `compiler.semcore::Repr::RVsa` | ctor | `lib/compiler/semcore.myc:3544` | `RVsa(Bytes, Binary{32}, SparsityC)` | — | Empirical/Declared |
| `compiler.semcore::Repr::RSeq` | ctor | `lib/compiler/semcore.myc:3545` | `RSeq(Repr, Binary{32})` | — | Empirical/Declared |
| `compiler.semcore::Repr::RFloat` | ctor | `lib/compiler/semcore.myc:3546` | `RFloat(FloatW)` | — | Empirical/Declared |
| `compiler.semcore::Repr::RBytes` | ctor | `lib/compiler/semcore.myc:3547` | `RBytes` | — | Empirical/Declared |
| `compiler.semcore::TritK` | type | `lib/compiler/semcore.myc:3550` | `type TritK = TkNeg \| TkZero \| TkPos` | TritK: mirrors mycelium_core::Trit (value.rs). | Empirical/Declared |
| `compiler.semcore::TritK::TkNeg` | ctor | `lib/compiler/semcore.myc:3550` | `TkNeg` | — | Empirical/Declared |
| `compiler.semcore::TritK::TkPos` | ctor | `lib/compiler/semcore.myc:3550` | `TkPos` | — | Empirical/Declared |
| `compiler.semcore::TritK::TkZero` | ctor | `lib/compiler/semcore.myc:3550` | `TkZero` | — | Empirical/Declared |
| `compiler.semcore::Payload` | type | `lib/compiler/semcore.myc:3554` | `type Payload = PlBits(Vec[Binary{1}]) \| PlTrits(Vec[TritK]) \| PlBytes(Bytes)` | Payload: mirrors mycelium_core::Payload (value.rs) — the wild-free-constructible subset only (FLAG-semcore-23). `PlBytes(Bytes)` holds the byte-vector directly (the `.myc` Bytes primitive). | Empirical/Declared |
| `compiler.semcore::Payload::PlBits` | ctor | `lib/compiler/semcore.myc:3554` | `PlBits(Vec[Binary{1}])` | — | Empirical/Declared |
| `compiler.semcore::Payload::PlBytes` | ctor | `lib/compiler/semcore.myc:3554` | `PlBytes(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::Payload::PlTrits` | ctor | `lib/compiler/semcore.myc:3554` | `PlTrits(Vec[TritK])` | — | Empirical/Declared |
| `compiler.semcore::Meta` | type | `lib/compiler/semcore.myc:3557` | `type Meta = MtExactRoot` | Meta: mirrors mycelium_core::Meta minimally (FLAG-semcore-24) — the single `exact(Root)` form. | Empirical/Declared |
| `compiler.semcore::Meta::MtExactRoot` | ctor | `lib/compiler/semcore.myc:3557` | `MtExactRoot` | — | Empirical/Declared |
| `compiler.semcore::Value` | type | `lib/compiler/semcore.myc:3560` | `type Value = Val(Repr, Payload, Meta)` | Value: mirrors mycelium_core::Value (value.rs) — the (repr, payload, meta) triple. | Empirical/Declared |
| `compiler.semcore::Value::Val` | ctor | `lib/compiler/semcore.myc:3560` | `Val(Repr, Payload, Meta)` | — | Empirical/Declared |
| `compiler.semcore::KFnSig` | type | `lib/compiler/semcore.myc:3564` | `type KFnSig = KFS(Binary{32}, Vec[FieldTyRef], FieldTyRef)` | KFnSig: mirrors mycelium_core::FnSig (data.rs) — (arity, params, ret). Spelled `KFnSig`/`KFS` to avoid the SURFACE `FnSig`/`FS` mirror already present (FLAG-semcore-22). | Empirical/Declared |
| `compiler.semcore::KFnSig::KFS` | ctor | `lib/compiler/semcore.myc:3564` | `KFS(Binary{32}, Vec[FieldTyRef], FieldTyRef)` | — | Empirical/Declared |
| `compiler.semcore::FieldTyRef` | type | `lib/compiler/semcore.myc:3567` | `type FieldTyRef = FtRepr(Repr) \| FtData(Bytes) \| FtFn(KFnSig)` | FieldTyRef: mirrors mycelium_core::FieldTyRef (data.rs). | Empirical/Declared |
| `compiler.semcore::FieldTyRef::FtData` | ctor | `lib/compiler/semcore.myc:3567` | `FtData(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::FieldTyRef::FtFn` | ctor | `lib/compiler/semcore.myc:3567` | `FtFn(KFnSig)` | — | Empirical/Declared |
| `compiler.semcore::FieldTyRef::FtRepr` | ctor | `lib/compiler/semcore.myc:3567` | `FtRepr(Repr)` | — | Empirical/Declared |
| `compiler.semcore::FieldSpec` | type | `lib/compiler/semcore.myc:3570` | `type FieldSpec = FsRepr(Repr) \| FsData(Bytes) \| FsFn(Binary{32}, KFnSig)` | FieldSpec: mirrors mycelium_core::FieldSpec (data.rs). | Empirical/Declared |
| `compiler.semcore::FieldSpec::FsData` | ctor | `lib/compiler/semcore.myc:3570` | `FsData(Bytes)` | — | Empirical/Declared |
| `compiler.semcore::FieldSpec::FsFn` | ctor | `lib/compiler/semcore.myc:3570` | `FsFn(Binary{32}, KFnSig)` | — | Empirical/Declared |
| `compiler.semcore::FieldSpec::FsRepr` | ctor | `lib/compiler/semcore.myc:3570` | `FsRepr(Repr)` | — | Empirical/Declared |
| `compiler.semcore::scalar_kind` | fn | `lib/compiler/semcore.myc:3576` | `fn scalar_kind(s: Scalar) => ScalarK` | scalar_kind (elab.rs:1043) — Scalar -> ScalarKind. Boundary-independent (lands first, DN-26 SS10); its tag twin `scalar_tag` (mono.rs) already landed increment 4. | Empirical/Declared |
| `compiler.semcore::sparsity_class` | fn | `lib/compiler/semcore.myc:3581` | `fn sparsity_class(sp: Sparsity) => SparsityC` | sparsity_class (elab.rs:1053) — Sparsity -> SparsityClass (the Sparse arm is a bare max_active passthrough). Boundary-independent; its tag twin `vsa_sp_tag` already landed increment 4. | Empirical/Declared |
| `compiler.semcore::bin_bits_at` | fn | `lib/compiler/semcore.myc:3588` | `fn bin_bits_at(s: Bytes, i: Binary{32}, n: Binary{32}) => Vec[Binary{1}]` | lit_value (elab.rs:102) — a representation literal's L0 Value. Bin/Trit/Str + refusals ported faithfully; LBytes/LFloat DEFERRED (FLAG-semcore-25). Never-silent on every fallible arm (G2). bin_bits: filter the digit/`_` string to its 0/1 bits, MSB-first (Rust `chars().filter(\|c\| c=='0'\|\|c=='1').map(\|c\| c=='1')`). | Empirical/Declared |
| `compiler.semcore::bin_bits` | fn | `lib/compiler/semcore.myc:3601` | `fn bin_bits(s: Bytes) => Vec[Binary{1}]` | — | Empirical/Declared |
| `compiler.semcore::bits_len` | fn | `lib/compiler/semcore.myc:3603` | `fn bits_len(v: Vec[Binary{1}]) => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::trit_of` | fn | `lib/compiler/semcore.myc:3608` | `fn trit_of(ch: Bytes) => Result[TritK, Bytes]` | trit_of: one wire glyph -> a trit; every non-{+,0,-} char is an explicit error (Rust maps EVERY char, no filtering — a stray char is a never-silent residual, G2). | Empirical/Declared |
| `compiler.semcore::trit_trits_at` | fn | `lib/compiler/semcore.myc:3614` | `fn trit_trits_at(s: Bytes, i: Binary{32}, n: Binary{32}) => Result[Vec[TritK], Bytes]` | — | Empirical/Declared |
| `compiler.semcore::trits_len` | fn | `lib/compiler/semcore.myc:3626` | `fn trits_len(v: Vec[TritK]) => Binary{32}` | — | Empirical/Declared |
| `compiler.semcore::lit_value` | fn | `lib/compiler/semcore.myc:3631` | `fn lit_value(l: Literal) => Result[Value, Bytes]` | lower-bound (`width == 0`) refused below, matching `Value::new`/`check_well_formed`; the `MAX_DIM` UPPER bound is NOT ported this wave (FLAG-semcore-29 — an honest, intentional cut). | Empirical/Declared |
| `compiler.semcore::type_repr` | fn | `lib/compiler/semcore.myc:3659` | `fn type_repr(t: TypeRef) => Result[Repr, Bytes]` | type_repr (elab.rs:244) — surface `TypeRef` -> kernel `Repr` (swap targets). Only representation types resolve; named/data/Substrate/Fn/Tuple/Ambient/width-var are explicit refusals. Note the Vsa arm canonicalizes the surface model id (`vsa_kernel_model_id`) -- matching the Rust oracle, and DISTINCT from `field_spec`/`ty_to_repr` below (whose checked `Ty` already carries the canonical id). | Empirical/Declared |
| `compiler.semcore::ty_to_repr` | fn | `lib/compiler/semcore.myc:3687` | `fn ty_to_repr(ty: Ty) => Option[Repr]` | ty_to_repr (elab.rs:982) — a representation checked `Ty` -> kernel `Repr`; `None` for any non-representation type (never a half-elaborated artifact -- G2/VR-5). Vsa uses the model DIRECTLY (the checked `Ty` already holds the canonical kernel id -- no re-canonicalization, matching Rust). | Empirical/Declared |
| `compiler.semcore::ty_to_field_ty_ref` | fn | `lib/compiler/semcore.myc:3705` | `fn ty_to_field_ty_ref(ty: Ty) => Option[FieldTyRef]` | ty_to_field_ty_ref (elab.rs:1023) — a v0 `Ty` -> a `FieldTyRef` signature leaf: a monomorphic `Data` reference, a nested `Fn` signature, or (the `_` arm) a `Repr` leaf via `ty_to_repr`. `None` for anything with no monomorphic form (mirrors `field_spec`'s staging -- G2/VR-5). | Empirical/Declared |
| `compiler.semcore::field_spec` | fn | `lib/compiler/semcore.myc:3723` | `fn field_spec(ty: Ty) => Option[FieldSpec]` | field_spec (elab.rs:923) — a checked `Ty` field -> a build-time `FieldSpec`; `None` when the field has no monomorphic kernel form (a width-var, a generic `Data`, a `Var`, a `Substrate`, or an unresolvable `Fn` signature leaf) -- staged, never half-encoded (G2/VR-5). | Empirical/Declared |
| `compiler.semcore::join_dot` | fn | `lib/compiler/semcore.myc:3747` | `fn join_dot(segs: Vec[Bytes]) => Bytes` | policy_name_preimage (elab.rs:344 preimage) — the domain-separated PREIMAGE of `policy_name_ref` (`policy-name.v0:<dotted>`). The hashing step is DEFERRED (FLAG-semcore-27). `join_dot` restates `policy.0.join(".")`. | Empirical/Declared |
| `compiler.semcore::policy_name_preimage` | fn | `lib/compiler/semcore.myc:3756` | `fn policy_name_preimage(p: Path) => Bytes` | — | Empirical/Declared |
| `compiler.semcore::Exports` | type | `lib/compiler/semcore.myc:3815` | `type Exports = Ex(Vec[Pair[Bytes, DataInfo]], Vec[Pair[Bytes, FnDecl]], Vec[Pair[Bytes, TraitInfo]], Vec[Pair[Bytes, Bool]])` | Exports: mirrors checkty.rs::Exports (1045-1056) as FOUR FLAG-semcore-4 assoc lists (no map primitive, M-982) keyed by the qualified (dot-joined) name STRING — `types`/`fns`/`traits` per kind, `declared` the ALL-declared-names-with-pub-ness table `resolve_imports` distinguishes "unknown" from "private" against. New ctor `Ex` (FLAG-semcore-2 scheme; checked, no collision). | Empirical/Declared |
| `compiler.semcore::Exports::Ex` | ctor | `lib/compiler/semcore.myc:3815` | `Ex(Vec[Pair[Bytes, DataInfo]], Vec[Pair[Bytes, FnDecl]], Vec[Pair[Bytes, TraitInfo]], Vec[Pair[Bytes, Bool]])` | — | Empirical/Declared |
| `compiler.semcore::ex_types` | fn | `lib/compiler/semcore.myc:3817` | `fn ex_types(e: Exports) => Vec[Pair[Bytes, DataInfo]]` | — | Empirical/Declared |
| `compiler.semcore::ex_fns` | fn | `lib/compiler/semcore.myc:3820` | `fn ex_fns(e: Exports) => Vec[Pair[Bytes, FnDecl]]` | — | Empirical/Declared |
| `compiler.semcore::ex_traits` | fn | `lib/compiler/semcore.myc:3823` | `fn ex_traits(e: Exports) => Vec[Pair[Bytes, TraitInfo]]` | — | Empirical/Declared |
| `compiler.semcore::ex_declared` | fn | `lib/compiler/semcore.myc:3826` | `fn ex_declared(e: Exports) => Vec[Pair[Bytes, Bool]]` | — | Empirical/Declared |
| `compiler.semcore::exports_types_lookup` | fn | `lib/compiler/semcore.myc:3832` | `fn exports_types_lookup(t: Vec[Pair[Bytes, DataInfo]], qual: Bytes) => Option[DataInfo]` | exports_types_lookup / exports_fns_lookup / exports_traits_lookup / exports_declared_lookup: the FLAG-semcore-4 linear scan over each table, keyed by the flat qualified-name `Bytes` string (four duplicated pairs rather than a generic dictionary — FLAG-semcore-5 precedent, KISS/YAGNI). | Empirical/Declared |
| `compiler.semcore::exports_fns_lookup` | fn | `lib/compiler/semcore.myc:3838` | `fn exports_fns_lookup(t: Vec[Pair[Bytes, FnDecl]], qual: Bytes) => Option[FnDecl]` | — | Empirical/Declared |
| `compiler.semcore::exports_traits_lookup` | fn | `lib/compiler/semcore.myc:3844` | `fn exports_traits_lookup(t: Vec[Pair[Bytes, TraitInfo]], qual: Bytes) => Option[TraitInfo]` | — | Empirical/Declared |
| `compiler.semcore::exports_declared_lookup` | fn | `lib/compiler/semcore.myc:3850` | `fn exports_declared_lookup(t: Vec[Pair[Bytes, Bool]], qual: Bytes) => Option[Bool]` | — | Empirical/Declared |
| `compiler.semcore::exports_has_pub` | fn | `lib/compiler/semcore.myc:3857` | `fn exports_has_pub(exports: Exports, qual: Bytes) => Bool` | exports_has_pub (checkty.rs 1535-1537): is `qual` declared AND `pub`? | Empirical/Declared |
| `compiler.semcore::NoduleImports` | type | `lib/compiler/semcore.myc:3866` | `type NoduleImports = NI(Vec[Pair[Bytes, DataInfo]], Vec[Pair[Bytes, FnDecl]], Vec[Pair[Bytes, TraitInfo]], Vec[Bytes])` | NoduleImports: mirrors checkty.rs::NoduleImports (1064-1075). `ambiguous` is a Vec[Bytes] SET (membership via `names_contains`) standing in for `BTreeSet<String>` (the same convention affine.rs FLAG-semcore-6/-18 established for accumulator sets). | Empirical/Declared |
| `compiler.semcore::NoduleImports::NI` | ctor | `lib/compiler/semcore.myc:3866` | `NI(Vec[Pair[Bytes, DataInfo]], Vec[Pair[Bytes, FnDecl]], Vec[Pair[Bytes, TraitInfo]], Vec[Bytes])` | — | Empirical/Declared |
| `compiler.semcore::ni_types` | fn | `lib/compiler/semcore.myc:3868` | `fn ni_types(n: NoduleImports) => Vec[Pair[Bytes, DataInfo]]` | — | Empirical/Declared |
| `compiler.semcore::ni_fns` | fn | `lib/compiler/semcore.myc:3871` | `fn ni_fns(n: NoduleImports) => Vec[Pair[Bytes, FnDecl]]` | — | Empirical/Declared |
| `compiler.semcore::ni_traits` | fn | `lib/compiler/semcore.myc:3874` | `fn ni_traits(n: NoduleImports) => Vec[Pair[Bytes, TraitInfo]]` | — | Empirical/Declared |
| `compiler.semcore::ni_ambiguous` | fn | `lib/compiler/semcore.myc:3877` | `fn ni_ambiguous(n: NoduleImports) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.semcore::nodule_imports_empty` | fn | `lib/compiler/semcore.myc:3881` | `fn nodule_imports_empty() => NoduleImports` | — | Empirical/Declared |
| `compiler.semcore::assoc_remove_datainfo` | fn | `lib/compiler/semcore.myc:3886` | `fn assoc_remove_datainfo(t: Vec[Pair[Bytes, DataInfo]], k: Bytes) => Vec[Pair[Bytes, DataInfo]]` | assoc_remove_\*/assoc_set_\*: `BTreeMap::insert` (replace-if-present) over each of the three per-kind tables — remove any existing pair for `k`, then cons the new one (order-insensitive, an assoc-list map; the FLAG-semcore-4 convention, three duplicated pairs per FLAG-semcore-5). | Empirical/Declared |
| `compiler.semcore::assoc_set_datainfo` | fn | `lib/compiler/semcore.myc:3895` | `fn assoc_set_datainfo(t: Vec[Pair[Bytes, DataInfo]], k: Bytes, v: DataInfo) => Vec[Pair[Bytes, DataInfo]]` | — | Empirical/Declared |
| `compiler.semcore::assoc_remove_fndecl` | fn | `lib/compiler/semcore.myc:3898` | `fn assoc_remove_fndecl(t: Vec[Pair[Bytes, FnDecl]], k: Bytes) => Vec[Pair[Bytes, FnDecl]]` | — | Empirical/Declared |
| `compiler.semcore::assoc_set_fndecl` | fn | `lib/compiler/semcore.myc:3907` | `fn assoc_set_fndecl(t: Vec[Pair[Bytes, FnDecl]], k: Bytes, v: FnDecl) => Vec[Pair[Bytes, FnDecl]]` | — | Empirical/Declared |
| `compiler.semcore::assoc_remove_traitinfo` | fn | `lib/compiler/semcore.myc:3910` | `fn assoc_remove_traitinfo(t: Vec[Pair[Bytes, TraitInfo]], k: Bytes) => Vec[Pair[Bytes, TraitInfo]]` | — | Empirical/Declared |
| `compiler.semcore::assoc_set_traitinfo` | fn | `lib/compiler/semcore.myc:3919` | `fn assoc_set_traitinfo(t: Vec[Pair[Bytes, TraitInfo]], k: Bytes, v: TraitInfo) => Vec[Pair[Bytes, TraitInfo]]` | — | Empirical/Declared |
| `compiler.semcore::names_remove` | fn | `lib/compiler/semcore.myc:3924` | `fn names_remove(xs: Vec[Bytes], n: Bytes) => Vec[Bytes]` | names_remove: drop every occurrence of `n` from a `Vec[Bytes]` SET (the `ambiguous.remove` counterpart for a bare-name set, not a keyed table). | Empirical/Declared |
| `compiler.semcore::insert_export` | fn | `lib/compiler/semcore.myc:3938` | `fn insert_export(imp: NoduleImports, exports: Exports, qual: Bytes, simple: Bytes) => NoduleImports` | insert_export (checkty.rs 1560-1571): insert the export `qual` into `imp` under `simple`. The oracle's OWN invariant comment says exactly one of the three export tables holds `qual` — but this port still tries all three (arm-for-arm), since nothing in `Exports`'s SHAPE enforces that invariant structurally (a caller-supplied fixture that violates it gets the same multi-table write the oracle's own three independent `if let`s would produce). | Empirical/Declared |
| `compiler.semcore::remove_import` | fn | `lib/compiler/semcore.myc:3954` | `fn remove_import(imp: NoduleImports, simple: Bytes) => NoduleImports` | remove_import (checkty.rs 1574-1580): drop any binding for `simple` across all three per-kind tables (the glob-vs-glob-ambiguity demotion) — `ambiguous` is untouched (matching the oracle). | Empirical/Declared |
| `compiler.semcore::qualify` | fn | `lib/compiler/semcore.myc:3958` | `fn qualify(path: Path, name: Bytes) => Bytes` | qualify (checkty.rs 1286-1292): `path.0.is_empty() ? name : path.0.join(".") + "." + name`. | Empirical/Declared |
| `compiler.semcore::bytes_starts_with` | fn | `lib/compiler/semcore.myc:3969` | `fn bytes_starts_with(s: Bytes, prefix: Bytes) => Bool` | bytes_starts_with / bytes_contains_char(_at): small string primitives NOT already in this file — needed because `direct_child`/`split_last_seg` operate on the FLAT qualified-name string (the `Exports` table's key shape), unlike every prior string helper here (which walks Path SEGMENTS, e.g. `join_dot`). Same byte-at-a-time idiom as `count_bindigits_at`/`filter_bindigits_at` (`bytes_slice` + `bytes_eq`, source-length-bounded direct-tail recursion). | Empirical/Declared |
| `compiler.semcore::bytes_contains_char_at` | fn | `lib/compiler/semcore.myc:3975` | `fn bytes_contains_char_at(s: Bytes, ch: Bytes, i: Binary{32}, n: Binary{32}) => Bool` | — | Empirical/Declared |
| `compiler.semcore::bytes_contains_char` | fn | `lib/compiler/semcore.myc:3984` | `fn bytes_contains_char(s: Bytes, ch: Bytes) => Bool` | — | Empirical/Declared |
| `compiler.semcore::direct_child` | fn | `lib/compiler/semcore.myc:3989` | `fn direct_child(prefix: Bytes, qual: Bytes) => Option[Bytes]` | direct_child (checkty.rs 1542-1550): is `qual` exactly ONE segment under `prefix`? `Some(simple)` iff `qual == prefix + "." + simple` with `simple` itself containing no further `.`. | Empirical/Declared |
| `compiler.semcore::rev_bytes_acc` | fn | `lib/compiler/semcore.myc:4016` | `fn rev_bytes_acc(xs: Vec[Bytes], acc: Vec[Bytes]) => Vec[Bytes]` | split_last_seg_go / split_last_seg (checkty.rs 1553-1557): `path.0.split_last()` restated as an accumulate-then-reverse walk (the FLAG-parse list-building convention) — last segment plus the DOT-JOINED prefix of everything before it (`Option[Pair[Bytes, Bytes]]`, matching the oracle's `Option<(String, String)>` return shape exactly). | Empirical/Declared |
| `compiler.semcore::split_last_seg_go` | fn | `lib/compiler/semcore.myc:4022` | `fn split_last_seg_go(segs: Vec[Bytes], acc: Vec[Bytes]) => Option[Pair[Bytes, Vec[Bytes]]]` | — | Empirical/Declared |
| `compiler.semcore::split_last_seg` | fn | `lib/compiler/semcore.myc:4031` | `fn split_last_seg(path: Path) => Option[Pair[Bytes, Bytes]]` | — | Empirical/Declared |
| `compiler.semcore::GlobPassState` | type | `lib/compiler/semcore.myc:4041` | `type GlobPassState = GPS(NoduleImports, Vec[Bytes], Vec[Bytes])` | GlobPassState: the glob loop's threaded (imp, via_explicit, via_glob) triple (FLAG-semcore-6 value-threading convention — Rust's `&mut` accumulators restated as an explicit fold). | Empirical/Declared |
| `compiler.semcore::GlobPassState::GPS` | ctor | `lib/compiler/semcore.myc:4041` | `GPS(NoduleImports, Vec[Bytes], Vec[Bytes])` | — | Empirical/Declared |
| `compiler.semcore::resolve_imports_glob_quals` | fn | `lib/compiler/semcore.myc:4046` | `fn resolve_imports_glob_quals(exports: Exports, prefix: Bytes, decl: Vec[Pair[Bytes, Bool]], st: GlobPassState) => GlobPassState` | resolve_imports_glob_quals: the INNER loop over `exports.declared`'s entries for ONE glob item's `prefix` (checkty.rs's `for qual in exports.declared.keys()`). FLAG-semcore-31 keeps the (provably always-false, see the file-header FLAG) `via_explicit` check for structural fidelity. | Empirical/Declared |
| `compiler.semcore::resolve_imports_glob_items` | fn | `lib/compiler/semcore.myc:4071` | `fn resolve_imports_glob_items(exports: Exports, items: Vec[Item], st: GlobPassState) => GlobPassState` | resolve_imports_glob_items: the OUTER glob-pass loop over `nodule.items` (non-glob / non-use items are skipped — the `let Item::Use(UsePath{path, glob:true}) = item else { continue }` mirror). | Empirical/Declared |
| `compiler.semcore::ExplicitPassState` | type | `lib/compiler/semcore.myc:4084` | `type ExplicitPassState = EPS(NoduleImports, Vec[Bytes])` | ExplicitPassState: the explicit-pass threaded (imp, via_explicit) pair. | Empirical/Declared |
| `compiler.semcore::ExplicitPassState::EPS` | ctor | `lib/compiler/semcore.myc:4084` | `EPS(NoduleImports, Vec[Bytes])` | — | Empirical/Declared |
| `compiler.semcore::resolve_imports_explicit_items` | fn | `lib/compiler/semcore.myc:4090` | `fn resolve_imports_explicit_items(exports: Exports, site: Bytes, items: Vec[Item], st: ExplicitPassState) => Result[ExplicitPassState, Bytes]` | resolve_imports_explicit_items (checkty.rs's explicit-`use` loop, 1478-1531): the never-silent refusals in the oracle's EXACT order — unnamed path, un-qualified single-segment path, unknown name, private name, duplicate explicit import (FLAG-semcore-32: reason strings, not the oracle's full diagnostic text). | Empirical/Declared |
| `compiler.semcore::resolve_imports` | fn | `lib/compiler/semcore.myc:4127` | `fn resolve_imports(nod: Nodule, exports: Exports) => Result[NoduleImports, Bytes]` | resolve_imports (checkty.rs 1423-1533, top level): globs first (lowest precedence), then explicit `use`s (which shadow a glob-bound name) — Ok(NoduleImports) or the first refusal. | Empirical/Declared |
| `compiler.semcore::fuse_trait_name` | fn | `lib/compiler/semcore.myc:4156` | `fn fuse_trait_name() => Bytes` | fuse_trait_name: the fuse.rs::TRAIT_NAME single source of truth (a fn, not a bare literal repeated at each use site -- Law of Demeter). | Empirical/Declared |
| `compiler.semcore::fuse_join_sig` | fn | `lib/compiler/semcore.myc:4161` | `fn fuse_join_sig() => FnSig` | fuse_join_sig: fuse.rs::prelude's one `FnSig` -- `fn join(a: T, b: T) => T`. No method-level type params (the trait's OWN param `T` lives in `TraitInfo.params`, not here -- DN-58 §A.2's explicit- type-variable idiom, no implicit `Self`), no effects/budgets. | Empirical/Declared |
| `compiler.semcore::fuse_prelude` | fn | `lib/compiler/semcore.myc:4165` | `fn fuse_prelude() => TraitInfo` | fuse_prelude: fuse.rs::prelude (58-84) verbatim -- `TrInfo(name, params, sigs)`. | Empirical/Declared |
| `compiler.semcore::checkty_prelude` | fn | `lib/compiler/semcore.myc:4185` | `fn checkty_prelude() => DataInfo` | checkty_prelude: checkty.rs::prelude (600-618) -- the builtin `Bool = False \| True` registry seed `register_nodule_decls` inserts before `register_types` runs, so intra-nodule resolution of `Bool` is unchanged. | Empirical/Declared |
| `compiler.semcore::nodule_uses_fuse` | fn | `lib/compiler/semcore.myc:4191` | `fn nodule_uses_fuse(items: Vec[Item]) => Bool` | `impl Fuse[...] for ...`? (Never based on whether `fuse(a, b)` is CALLED -- the repr-type fast path in `check_fuse` never touches the trait registry at all; restated from the oracle's own comment.) | Empirical/Declared |
| `compiler.semcore::fuse_redeclare_err` | fn | `lib/compiler/semcore.myc:4207` | `fn fuse_redeclare_err() => Bytes` | fuse_redeclare_err: checkty.rs 1372-1378 / 1381-1387 -- the never-silent refusal for either branch that finds `Fuse` already present in `traits` (a user nodule trying to redeclare the built-in). Both oracle call sites use the SAME message text (FLAG-semcore-32: error TEXT is not differential- compared -- only Ok/Err CLASSIFICATION is; this file's established convention throughout STEP 4/5). | Empirical/Declared |
| `compiler.semcore::seed_fuse_trait` | fn | `lib/compiler/semcore.myc:4214` | `fn seed_fuse_trait(traits: Vec[TraitInfo], items: Vec[Item]) => Result[Vec[TraitInfo], Bytes]` | (already present, on EITHER branch) -- never a silent shadow of the built-in (G2/VR-5), mirroring how redeclaring `Bool` would collide in `types`. | Empirical/Declared |
| `compiler.semcore::fns_contains` | fn | `lib/compiler/semcore.myc:4228` | `fn fns_contains(fns: Vec[Pair[Bytes, FnDecl]], name: Bytes) => Bool` | fns_contains: the `fns.contains_key`-equivalent membership scan over the FLAG-semcore-4 assoc-list fn registry `Vec[Pair[Bytes, FnDecl]]`. | Empirical/Declared |
| `compiler.semcore::register_fns` | fn | `lib/compiler/semcore.myc:4239` | `fn register_fns(items: Vec[Item], acc: Vec[Pair[Bytes, FnDecl]]) => Result[Vec[Pair[Bytes, FnDecl]], Bytes]` | register_fns: checkty.rs 1392-1400 -- Pass 2, the fn-signature registration loop. Non-`Item::Fn` items are skipped (the `let Item::Fn(fd) = item else { continue }` mirror); each fn's OWN type-parameter names (`fnsig_tyvar_names`, the `sig.param_names()` mirror) must have no duplicate, then the fn NAME must be free in the registry so far -- the first refusal wins, `Ok` with the built registry otherwise. | Empirical/Declared |
| `compiler.semcore::NoduleRegs` | type | `lib/compiler/semcore.myc:4256` | `type NoduleRegs = NR(Vec[DataInfo], Vec[Pair[Bytes, FnDecl]], Vec[TraitInfo])` | NoduleRegs: mirrors checkty.rs::NoduleRegs (1336-1340) field-for-field (types, fns, traits). New ctor `NR` (FLAG-semcore-2 scheme; checked against every other family in this file, no collision). | Empirical/Declared |
| `compiler.semcore::NoduleRegs::NR` | ctor | `lib/compiler/semcore.myc:4256` | `NR(Vec[DataInfo], Vec[Pair[Bytes, FnDecl]], Vec[TraitInfo])` | — | Empirical/Declared |
| `compiler.semcore::nr_types` | fn | `lib/compiler/semcore.myc:4258` | `fn nr_types(r: NoduleRegs) => Vec[DataInfo]` | — | Empirical/Declared |
| `compiler.semcore::nr_fns` | fn | `lib/compiler/semcore.myc:4261` | `fn nr_fns(r: NoduleRegs) => Vec[Pair[Bytes, FnDecl]]` | — | Empirical/Declared |
| `compiler.semcore::nr_traits` | fn | `lib/compiler/semcore.myc:4264` | `fn nr_traits(r: NoduleRegs) => Vec[TraitInfo]` | — | Empirical/Declared |
| `compiler.semcore::register_nodule_decls` | fn | `lib/compiler/semcore.myc:4270` | `fn register_nodule_decls(nod: Nodule) => Result[NoduleRegs, Bytes]` | register_nodule_decls: checkty.rs 1348-1401, top level. Runs the four passes in the oracle's EXACT order -- types (seeded with the Bool prelude), traits, the conditional Fuse seed, then fns -- and returns Ok(NoduleRegs) or the FIRST refusal. | Empirical/Declared |
| `compiler.semcore::closure_field_ty` | fn | `lib/compiler/semcore.myc:4331` | `fn closure_field_ty(t: Ty) => Ty` | — | Empirical/Declared |
| `compiler.semcore::closure_param_ref` | fn | `lib/compiler/semcore.myc:4337` | `fn closure_param_ref(t: Ty) => TypeRef` | — | Empirical/Declared |
| `compiler.semcore::mangle_ty_in_ty` | fn | `lib/compiler/semcore.myc:4344` | `fn mangle_ty_in_ty(t: Ty) => Ty` | — | Empirical/Declared |
| `compiler.semcore::ty_to_source_base` | fn | `lib/compiler/semcore.myc:4362` | `fn ty_to_source_base(t: Ty) => BaseType` | — | Empirical/Declared |
| `compiler.semcore::ty_to_source_ref` | fn | `lib/compiler/semcore.myc:4377` | `fn ty_to_source_ref(t: Ty) => TypeRef` | — | Empirical/Declared |
| `compiler.semcore::ty_to_source_ref_list` | fn | `lib/compiler/semcore.myc:4382` | `fn ty_to_source_ref_list(ts: Vec[Ty]) => Vec[TypeRef]` | ty_to_source_ref_list: map a constructor's applied type args through ty_to_source_ref (the direct recursive walk this nodule uses throughout — no first-class-fn `map` primitive, FLAG-semcore-5's KISS/YAGNI precedent for avoiding a generic higher-order helper in a bounded increment). | Empirical/Declared |
| `compiler.semcore::ty_to_ref_base` | fn | `lib/compiler/semcore.myc:4388` | `fn ty_to_ref_base(t: Ty) => BaseType` | ty_to_ref_base: the MANGLED-NULLARY round-trip (an applied data type's arguments are baked into its mangled name, so it becomes a nullary `Named`) — used for a rewritten `FnDecl`/`Param`/ `Ascribe`'s emitted surface type. | Empirical/Declared |
| `compiler.semcore::ty_to_ref` | fn | `lib/compiler/semcore.myc:4403` | `fn ty_to_ref(t: Ty) => TypeRef` | — | Empirical/Declared |
| `compiler.semcore::ty_to_ref_tagged` | fn | `lib/compiler/semcore.myc:4407` | `fn ty_to_ref_tagged(t: Ty, guarantee: Option[Strength]) => TypeRef` | ty_to_ref_tagged: like ty_to_ref, but attaches the CALLER-supplied `guarantee` (the source declaration's own `@ g` — never derived from `t`, never merged across instantiations). | Empirical/Declared |
| `compiler.semcore::WorkItem` | type | `lib/compiler/semcore.myc:4412` | `type WorkItem = WiFn(Bytes, Vec[Ty], Vec[Width], Vec[Pair[Binary{32}, Bytes]], Vec[Pair[Binary{32}, Bytes]]) \| WiData(Bytes, Vec[Ty]) \| WiMethod(Bytes, Bytes, Ty)` | — | Empirical/Declared |
| `compiler.semcore::WorkItem::WiFn` | ctor | `lib/compiler/semcore.myc:4413` | `WiFn(Bytes, Vec[Ty], Vec[Width], Vec[Pair[Binary{32}, Bytes]], Vec[Pair[Binary{32}, Bytes]])` | — | Empirical/Declared |
| `compiler.semcore::WorkItem::WiData` | ctor | `lib/compiler/semcore.myc:4414` | `WiData(Bytes, Vec[Ty])` | — | Empirical/Declared |
| `compiler.semcore::WorkItem::WiMethod` | ctor | `lib/compiler/semcore.myc:4415` | `WiMethod(Bytes, Bytes, Ty)` | — | Empirical/Declared |
| `compiler.semcore::item_key` | fn | `lib/compiler/semcore.myc:4418` | `fn item_key(item: WorkItem) => Bytes` | item_key: the canonical dedup key of a work item — a kind-tagged mangled string. | Empirical/Declared |
| `compiler.semcore::LVal` | type | `lib/compiler/semcore.myc:4475` | `type LVal = LData(Bytes, Bytes, Vec[LVal]) \| LOpaque` | FLAG-semcore-36 (`LVal` -- the eval.rs::L1Value mirror, TRIMMED to what `try_match`'s ported fragment needs). `Data { ty, ctor, fields }` mirrors verbatim (`LData`, FLAG-semcore-2 `L`-prefix scheme extended: no collision against any family declared above). `Repr`/`Substrate`/`Fn` collapse into ONE nullary `LOpaque` (FLAG-semcore-35 explains why this is lossless for the ported arms). `binds` (Rust's `&mut Vec<(String, L1Value)>`, mutated by `push` -- APPEND order) is restated as the FLAG-semcore-6 value-threaded collapse: threaded through and RETURNED, and appended (not prepended, unlike the FLAG-semcore-13 assoc-list convention) via `bind_append` below, because `binds`' ORDER is part of the observable differential (`Vec` equality is order-sensitive) -- unlike `unify`'s substitution, whose FIRST-MATCH lookup semantics make prepend-order invisible. | Empirical/Declared |
| `compiler.semcore::LVal::LData` | ctor | `lib/compiler/semcore.myc:4475` | `LData(Bytes, Bytes, Vec[LVal])` | — | Empirical/Declared |
| `compiler.semcore::LVal::LOpaque` | ctor | `lib/compiler/semcore.myc:4475` | `LOpaque` | — | Empirical/Declared |
| `compiler.semcore::bind_append` | fn | `lib/compiler/semcore.myc:4481` | `fn bind_append(binds: Vec[Pair[Bytes, LVal]], n: Bytes, v: LVal) => Vec[Pair[Bytes, LVal]]` | bind_append: append one `(name, val)` binding to the END of `binds` -- mirrors Rust's `binds.push((n.clone(), val.clone()))` exactly (order-preserving; bounded by one pattern's own binder count, never source-input-driven, so no depth budget is warranted -- the same class of bounded structural recursion as `pat_append`/`ty_append` above). | Empirical/Declared |
| `compiler.semcore::ctorinfo_any_named` | fn | `lib/compiler/semcore.myc:4494` | `fn ctorinfo_any_named(ctors: Vec[CtorInfo], n: Bytes) => Bool` | lval_ident_is_ctor_of / ctorinfo_any_named: mirrors try_match's `Ident` guard `self.env.types.get(ty).is_some_and(\|d\| d.ctors.iter().any(\|c\| c.name == \*n))` over the ALREADY -ported `types_lookup`/`di_ctors`/`ci_name` registry accessors (increment 8, M-1013 STEP 3) -- no new registry machinery needed, only the `.any(name == n)` scan. | Empirical/Declared |
| `compiler.semcore::lval_ident_is_ctor_of` | fn | `lib/compiler/semcore.myc:4500` | `fn lval_ident_is_ctor_of(types: Vec[DataInfo], ty: Bytes, n: Bytes) => Bool` | — | Empirical/Declared |
| `compiler.semcore::lval_try_match` | fn | `lib/compiler/semcore.myc:4511` | `fn lval_try_match(types: Vec[DataInfo], pat: Pattern, val: LVal, binds: Vec[Pair[Bytes, LVal]]) => Result[Pair[Bool, Vec[Pair[Bytes, LVal]]], Bytes]` | lval_try_match: the ported fragment (mirrors eval.rs::Evaluator::try_match's `Wildcard`/`Ident`/ `Ctor`/`Tuple`/`Or` arms exactly; `Lit` refuses per FLAG-semcore-35). `site` is DROPPED (the FLAG-semcore-13 precedent: the differential asserts `Ok`/`Err` + the resulting `(bool, binds)`, never the message bytes) -- both the oracle's `L1Error` and this fn's `Bytes` message collapse to `Err(())` on the marshalling harness's `decode_result`. | Empirical/Declared |
| `compiler.semcore::lval_match_zip` | fn | `lib/compiler/semcore.myc:4539` | `fn lval_match_zip(types: Vec[DataInfo], subs: Vec[Pattern], fields: Vec[LVal], binds: Vec[Pair[Bytes, LVal]]) => Result[Pair[Bool, Vec[Pair[Bytes, LVal]]], Bytes]` | lval_match_zip: the `Ctor` arm's `for (sub, fv) in subs.iter().zip(fields.iter())` loop, restated as a fold that THREADS `binds` and short-circuits `Ok(false)` on the first sub-pattern mismatch -- mirrors Rust's `zip` early-stop-at-the-shorter-list semantics exactly (a length mismatch between `subs`/`fields` never occurs in a checked program; where it would, `zip` silently stops rather than erroring, and so does this fold, for byte-for-byte behavioral parity). | Empirical/Declared |

### compiler.substrate

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.substrate` | nodule | `lib/compiler/substrate.myc:13` | `nodule compiler.substrate` | Self-hosted port of the `Substrate` v0 value form's DETERMINISTIC surface (M-740 Stage 4; DN-26 §7.3 row 4). A deliberately NARROWED port of crates/mycelium-l1/src/substrate.rs — the shared-across-clone atomic-CAS use-once backstop has no pure-value analog (FLAG-substrate-1 below); everything else (provenance, explain, the consume-once transition along one threaded value, release, error/event formatting) is ported. | Empirical/Declared |
| `compiler.substrate::Option` | type | `lib/compiler/substrate.myc:108` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.substrate::Option::None` | ctor | `lib/compiler/substrate.myc:108` | `None` | — | Empirical/Declared |
| `compiler.substrate::Option::Some` | ctor | `lib/compiler/substrate.myc:108` | `Some(A)` | — | Empirical/Declared |
| `compiler.substrate::Result` | type | `lib/compiler/substrate.myc:109` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `compiler.substrate::Result::Err` | ctor | `lib/compiler/substrate.myc:109` | `Err(E)` | — | Empirical/Declared |
| `compiler.substrate::Result::Ok` | ctor | `lib/compiler/substrate.myc:109` | `Ok(A)` | — | Empirical/Declared |
| `compiler.substrate::Vec` | type | `lib/compiler/substrate.myc:110` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.substrate::Vec::Cons` | ctor | `lib/compiler/substrate.myc:110` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.substrate::Vec::Nil` | ctor | `lib/compiler/substrate.myc:110` | `Nil` | — | Empirical/Declared |
| `compiler.substrate::Pair` | type | `lib/compiler/substrate.myc:111` | `type Pair[A, B] = Pr(A, B)` | — | Empirical/Declared |
| `compiler.substrate::Pair::Pr` | ctor | `lib/compiler/substrate.myc:111` | `Pr(A, B)` | — | Empirical/Declared |
| `compiler.substrate::empty_bytes` | fn | `lib/compiler/substrate.myc:114` | `fn empty_bytes() => Bytes` | — | Empirical/Declared |
| `compiler.substrate::digit_bytes` | fn | `lib/compiler/substrate.myc:129` | `fn digit_bytes(d: Binary{32}) => Bytes` | — | Empirical/Declared |
| `compiler.substrate::itoa_acc` | fn | `lib/compiler/substrate.myc:172` | `fn itoa_acc(n: Binary{32}, acc: Vec[Bytes]) => Vec[Bytes]` | Worked example, n = 123: iter 1 -> rem=3, acc=[3];  n=12 iter 2 -> rem=2, acc=[2,3]; n=1 iter 3 -> rem=1, acc=[1,2,3]; n=0, stop. acc is already MOST-significant-digit-first ([1,2,3] = "123") — no `rev_acc` needed: each digit `Cons`ed later is MORE significant, so front-insertion naturally produces place-value order. | Empirical/Declared |
| `compiler.substrate::concat_bytes_list` | fn | `lib/compiler/substrate.myc:184` | `fn concat_bytes_list(xs: Vec[Bytes]) => Bytes` | concat_bytes_list: join a Vec[Bytes] in order. Non-tail, bounded by the same small digit-count (the join_segs / ast.myc precedent for a non-tail recursion bounded by a handful of items, not source length). | Empirical/Declared |
| `compiler.substrate::itoa` | fn | `lib/compiler/substrate.myc:192` | `fn itoa(n: Binary{32}) => Bytes` | itoa: the decimal ASCII text of a non-negative Binary{32} value ("0" for zero; no leading zero otherwise, matching Rust's `{}` Display for an unsigned integer). | Empirical/Declared |
| `compiler.substrate::SubstrateProvenance` | type | `lib/compiler/substrate.myc:201` | `type SubstrateProvenance = Prov(Bytes, Bytes)` | — | Empirical/Declared |
| `compiler.substrate::SubstrateProvenance::Prov` | ctor | `lib/compiler/substrate.myc:201` | `Prov(Bytes, Bytes)` | — | Empirical/Declared |
| `compiler.substrate::prov_acquired_via` | fn | `lib/compiler/substrate.myc:203` | `fn prov_acquired_via(p: SubstrateProvenance) => Bytes` | — | Empirical/Declared |
| `compiler.substrate::prov_site` | fn | `lib/compiler/substrate.myc:206` | `fn prov_site(p: SubstrateProvenance) => Bytes` | — | Empirical/Declared |
| `compiler.substrate::prov_new` | fn | `lib/compiler/substrate.myc:209` | `fn prov_new(acquired_via: Bytes, site: Bytes) => SubstrateProvenance` | — | Empirical/Declared |
| `compiler.substrate::SubstrateHandle` | type | `lib/compiler/substrate.myc:215` | `type SubstrateHandle = SH(Bytes, Binary{32}, SubstrateProvenance, Bool)` | — | Empirical/Declared |
| `compiler.substrate::SubstrateHandle::SH` | ctor | `lib/compiler/substrate.myc:215` | `SH(Bytes, Binary{32}, SubstrateProvenance, Bool)` | — | Empirical/Declared |
| `compiler.substrate::sh_tag` | fn | `lib/compiler/substrate.myc:217` | `fn sh_tag(h: SubstrateHandle) => Bytes` | — | Empirical/Declared |
| `compiler.substrate::sh_id` | fn | `lib/compiler/substrate.myc:220` | `fn sh_id(h: SubstrateHandle) => Binary{32}` | — | Empirical/Declared |
| `compiler.substrate::sh_provenance` | fn | `lib/compiler/substrate.myc:223` | `fn sh_provenance(h: SubstrateHandle) => SubstrateProvenance` | — | Empirical/Declared |
| `compiler.substrate::sh_is_consumed` | fn | `lib/compiler/substrate.myc:226` | `fn sh_is_consumed(h: SubstrateHandle) => Bool` | — | Empirical/Declared |
| `compiler.substrate::sh_eq` | fn | `lib/compiler/substrate.myc:232` | `fn sh_eq(a: SubstrateHandle, b: SubstrateHandle) => Bool` | sh_eq: identity equality (mirrors substrate.rs::PartialEq for SubstrateHandle — "two handles are the same resource iff their ids are equal"; `provenance` is invariant per id and `consumed` is state over an identity, not part of it, exactly as the Rust doc comment reasons). | Empirical/Declared |
| `compiler.substrate::acquire` | fn | `lib/compiler/substrate.myc:240` | `fn acquire(tag: Bytes, provenance: SubstrateProvenance, next_id: Binary{32}) => Pair[SubstrateHandle, Binary{32}]` | acquire: mint a fresh Live handle (mirrors substrate.rs::SubstrateHandle::acquire) — see FLAG-substrate-2 for the explicit next_id threading this value-language port requires in place of a process-global atomic counter. Returns the minted handle paired with the next-id to use for a SUBSEQUENT acquisition (mirrors the oracle's real invariant: every acquire mints a fresh, distinct identity). | Empirical/Declared |
| `compiler.substrate::acquired_handle` | fn | `lib/compiler/substrate.myc:243` | `fn acquired_handle(p: Pair[SubstrateHandle, Binary{32}]) => SubstrateHandle` | — | Empirical/Declared |
| `compiler.substrate::acquired_next_id` | fn | `lib/compiler/substrate.myc:246` | `fn acquired_next_id(p: Pair[SubstrateHandle, Binary{32}]) => Binary{32}` | — | Empirical/Declared |
| `compiler.substrate::explain` | fn | `lib/compiler/substrate.myc:252` | `fn explain(h: SubstrateHandle) => Bytes` | explain: a never-silent, one-line EXPLAIN description (house rule 2 — no black boxes; mirrors substrate.rs::SubstrateHandle::explain BYTE-FOR-BYTE — its Rust format string is pure ASCII, so no FLAG-substrate-4 narrowing applies here). | Empirical/Declared |
| `compiler.substrate::SubstrateError` | type | `lib/compiler/substrate.myc:264` | `type SubstrateError = AlreadyConsumed(Bytes, Binary{32})` | — | Empirical/Declared |
| `compiler.substrate::SubstrateError::AlreadyConsumed` | ctor | `lib/compiler/substrate.myc:264` | `AlreadyConsumed(Bytes, Binary{32})` | — | Empirical/Declared |
| `compiler.substrate::se_tag` | fn | `lib/compiler/substrate.myc:266` | `fn se_tag(e: SubstrateError) => Bytes` | — | Empirical/Declared |
| `compiler.substrate::se_id` | fn | `lib/compiler/substrate.myc:269` | `fn se_id(e: SubstrateError) => Binary{32}` | — | Empirical/Declared |
| `compiler.substrate::se_display` | fn | `lib/compiler/substrate.myc:276` | `fn se_display(e: SubstrateError) => Bytes` | se_display: an ASCII-safe structural mirror of substrate.rs::SubstrateError's Display impl — FLAG-substrate-4 (the real Display prose uses an em dash; ported here as an ASCII-only equivalent carrying the SAME two fields in the SAME order — only structural/field content is differential-tested, against a matching Rust-side mirror function, not the live Display impl). | Empirical/Declared |
| `compiler.substrate::try_consume` | fn | `lib/compiler/substrate.myc:288` | `fn try_consume(h: SubstrateHandle) => Result[SubstrateHandle, SubstrateError]` | — | Empirical/Declared |
| `compiler.substrate::ReleaseEvent` | type | `lib/compiler/substrate.myc:297` | `type ReleaseEvent = RE(Bytes, Binary{32}, Bytes)` | — | Empirical/Declared |
| `compiler.substrate::ReleaseEvent::RE` | ctor | `lib/compiler/substrate.myc:297` | `RE(Bytes, Binary{32}, Bytes)` | — | Empirical/Declared |
| `compiler.substrate::re_tag` | fn | `lib/compiler/substrate.myc:299` | `fn re_tag(e: ReleaseEvent) => Bytes` | — | Empirical/Declared |
| `compiler.substrate::re_id` | fn | `lib/compiler/substrate.myc:302` | `fn re_id(e: ReleaseEvent) => Binary{32}` | — | Empirical/Declared |
| `compiler.substrate::re_site` | fn | `lib/compiler/substrate.myc:305` | `fn re_site(e: ReleaseEvent) => Bytes` | — | Empirical/Declared |
| `compiler.substrate::re_display` | fn | `lib/compiler/substrate.myc:311` | `fn re_display(e: ReleaseEvent) => Bytes` | re_display: an ASCII-safe structural mirror of substrate.rs::ReleaseEvent's Display impl — FLAG-substrate-4 (same allowance as se_display above: same three fields, same order, shorter ASCII-only prose; differential-tested against a matching Rust-side mirror, not the live Display). | Empirical/Declared |
| `compiler.substrate::release` | fn | `lib/compiler/substrate.myc:326` | `fn release(h: SubstrateHandle, site: Bytes) => Option[ReleaseEvent]` | — | Empirical/Declared |

### compiler.token

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.token` | nodule | `lib/compiler/token.myc:11` | `nodule compiler.token` | Self-hosted L1 token kinds + source positions (M-740 Stage 1; DN-26 §7.3 row 1). A faithful port of crates/mycelium-l1/src/token.rs plus the token-relevant slice of error.rs (the small token<->error 2-cycle, DN-26 §7.1) — this nodule carries only the token/position DATA and the `keyword` classifier; the scanning state machine is `compiler.lex` (lex.myc, sibling nodule). | Empirical/Declared |
| `compiler.token::Option` | type | `lib/compiler/token.myc:76` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.token::Option::None` | ctor | `lib/compiler/token.myc:76` | `None` | — | Empirical/Declared |
| `compiler.token::Option::Some` | ctor | `lib/compiler/token.myc:76` | `Some(A)` | — | Empirical/Declared |
| `compiler.token::Pos` | type | `lib/compiler/token.myc:81` | `type Pos = P(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.token::Pos::P` | ctor | `lib/compiler/token.myc:81` | `P(Binary{32}, Binary{32})` | — | Empirical/Declared |
| `compiler.token::pos_line` | fn | `lib/compiler/token.myc:83` | `fn pos_line(p: Pos) => Binary{32}` | — | Empirical/Declared |
| `compiler.token::pos_col` | fn | `lib/compiler/token.myc:86` | `fn pos_col(p: Pos) => Binary{32}` | — | Empirical/Declared |
| `compiler.token::ScalarTok` | type | `lib/compiler/token.myc:91` | `type ScalarTok = SF16 \| SBf16 \| SF32 \| SF64` | — | Empirical/Declared |
| `compiler.token::ScalarTok::SBf16` | ctor | `lib/compiler/token.myc:91` | `SBf16` | — | Empirical/Declared |
| `compiler.token::ScalarTok::SF16` | ctor | `lib/compiler/token.myc:91` | `SF16` | — | Empirical/Declared |
| `compiler.token::ScalarTok::SF32` | ctor | `lib/compiler/token.myc:91` | `SF32` | — | Empirical/Declared |
| `compiler.token::ScalarTok::SF64` | ctor | `lib/compiler/token.myc:91` | `SF64` | — | Empirical/Declared |
| `compiler.token::StrengthTok` | type | `lib/compiler/token.myc:97` | `type StrengthTok = GExact \| GProven \| GEmpirical \| GDeclared` | — | Empirical/Declared |
| `compiler.token::StrengthTok::GDeclared` | ctor | `lib/compiler/token.myc:97` | `GDeclared` | — | Empirical/Declared |
| `compiler.token::StrengthTok::GEmpirical` | ctor | `lib/compiler/token.myc:97` | `GEmpirical` | — | Empirical/Declared |
| `compiler.token::StrengthTok::GExact` | ctor | `lib/compiler/token.myc:97` | `GExact` | — | Empirical/Declared |
| `compiler.token::StrengthTok::GProven` | ctor | `lib/compiler/token.myc:97` | `GProven` | — | Empirical/Declared |
| `compiler.token::Tok` | type | `lib/compiler/token.myc:104` | `type Tok = Nodule \| Phylum \| Colony \| Hypha \| Fuse \| Mesh \| Graft \| Cyst \| Xloc \| Forage \| Backbone \| Tier \| Reclaim \| Consume \| Grow \| Derive \| Use \| Pub \| Type \| Trait \| Impl \| Fn \| Matured \| Thaw \| Let \| In \| If \| Then \| Else \| Match \| For \| Swap \| Default \| Paradigm \| With \| Wild \| Spore \| To \| Policy \| Lambda \| Object \| Via \| Lower \| KwBinary \| KwTernary \| KwDense \| Vsa \| BinShort \| TernShort \| EmbShort \| HvecShort \| KwSeq \| KwBytes \| KwFloat \| KwSubstrate \| KwSparse \| Scalar(ScalarTok) \| Strength(StrengthTok) \| Ident(Bytes) \| BinLit(Bytes) \| BytesLit(Bytes) \| TritLit(Bytes) \| StrLit(Bytes) \| Int(Bytes) \| FloatLit(Bytes) \| LParen \| RParen \| LBrace \| RBrace \| LBracket \| RBracket \| LAngle \| RAngle \| Shl \| Shr \| At \| AtStdSys \| Colon \| Comma \| Semi \| Dot \| Pipe \| Plus \| Minus \| Star \| Slash \| Percent \| Caret \| Amp \| AmpAmp \| Eq \| EqEq \| Arrow \| FatArrow \| Bang \| BangEq \| PipePipe \| Eof` | — | Empirical/Declared |
| `compiler.token::Tok::Colony` | ctor | `lib/compiler/token.myc:106` | `Colony` | — | Empirical/Declared |
| `compiler.token::Tok::Nodule` | ctor | `lib/compiler/token.myc:106` | `Nodule` | — | Empirical/Declared |
| `compiler.token::Tok::Phylum` | ctor | `lib/compiler/token.myc:106` | `Phylum` | — | Empirical/Declared |
| `compiler.token::Tok::Backbone` | ctor | `lib/compiler/token.myc:108` | `Backbone` | — | Empirical/Declared |
| `compiler.token::Tok::Cyst` | ctor | `lib/compiler/token.myc:108` | `Cyst` | — | Empirical/Declared |
| `compiler.token::Tok::Forage` | ctor | `lib/compiler/token.myc:108` | `Forage` | — | Empirical/Declared |
| `compiler.token::Tok::Fuse` | ctor | `lib/compiler/token.myc:108` | `Fuse` | — | Empirical/Declared |
| `compiler.token::Tok::Graft` | ctor | `lib/compiler/token.myc:108` | `Graft` | — | Empirical/Declared |
| `compiler.token::Tok::Hypha` | ctor | `lib/compiler/token.myc:108` | `Hypha` | — | Empirical/Declared |
| `compiler.token::Tok::Mesh` | ctor | `lib/compiler/token.myc:108` | `Mesh` | — | Empirical/Declared |
| `compiler.token::Tok::Reclaim` | ctor | `lib/compiler/token.myc:108` | `Reclaim` | — | Empirical/Declared |
| `compiler.token::Tok::Tier` | ctor | `lib/compiler/token.myc:108` | `Tier` | — | Empirical/Declared |
| `compiler.token::Tok::Xloc` | ctor | `lib/compiler/token.myc:108` | `Xloc` | — | Empirical/Declared |
| `compiler.token::Tok::Consume` | ctor | `lib/compiler/token.myc:110` | `Consume` | — | Empirical/Declared |
| `compiler.token::Tok::Derive` | ctor | `lib/compiler/token.myc:110` | `Derive` | — | Empirical/Declared |
| `compiler.token::Tok::Grow` | ctor | `lib/compiler/token.myc:110` | `Grow` | — | Empirical/Declared |
| `compiler.token::Tok::Fn` | ctor | `lib/compiler/token.myc:112` | `Fn` | — | Empirical/Declared |
| `compiler.token::Tok::Impl` | ctor | `lib/compiler/token.myc:112` | `Impl` | — | Empirical/Declared |
| `compiler.token::Tok::Matured` | ctor | `lib/compiler/token.myc:112` | `Matured` | — | Empirical/Declared |
| `compiler.token::Tok::Pub` | ctor | `lib/compiler/token.myc:112` | `Pub` | — | Empirical/Declared |
| `compiler.token::Tok::Thaw` | ctor | `lib/compiler/token.myc:112` | `Thaw` | — | Empirical/Declared |
| `compiler.token::Tok::Trait` | ctor | `lib/compiler/token.myc:112` | `Trait` | — | Empirical/Declared |
| `compiler.token::Tok::Type` | ctor | `lib/compiler/token.myc:112` | `Type` | — | Empirical/Declared |
| `compiler.token::Tok::Use` | ctor | `lib/compiler/token.myc:112` | `Use` | — | Empirical/Declared |
| `compiler.token::Tok::Else` | ctor | `lib/compiler/token.myc:113` | `Else` | — | Empirical/Declared |
| `compiler.token::Tok::For` | ctor | `lib/compiler/token.myc:113` | `For` | — | Empirical/Declared |
| `compiler.token::Tok::If` | ctor | `lib/compiler/token.myc:113` | `If` | — | Empirical/Declared |
| `compiler.token::Tok::In` | ctor | `lib/compiler/token.myc:113` | `In` | — | Empirical/Declared |
| `compiler.token::Tok::Let` | ctor | `lib/compiler/token.myc:113` | `Let` | — | Empirical/Declared |
| `compiler.token::Tok::Match` | ctor | `lib/compiler/token.myc:113` | `Match` | — | Empirical/Declared |
| `compiler.token::Tok::Swap` | ctor | `lib/compiler/token.myc:113` | `Swap` | — | Empirical/Declared |
| `compiler.token::Tok::Then` | ctor | `lib/compiler/token.myc:113` | `Then` | — | Empirical/Declared |
| `compiler.token::Tok::Default` | ctor | `lib/compiler/token.myc:114` | `Default` | — | Empirical/Declared |
| `compiler.token::Tok::Paradigm` | ctor | `lib/compiler/token.myc:114` | `Paradigm` | — | Empirical/Declared |
| `compiler.token::Tok::Policy` | ctor | `lib/compiler/token.myc:114` | `Policy` | — | Empirical/Declared |
| `compiler.token::Tok::Spore` | ctor | `lib/compiler/token.myc:114` | `Spore` | — | Empirical/Declared |
| `compiler.token::Tok::To` | ctor | `lib/compiler/token.myc:114` | `To` | — | Empirical/Declared |
| `compiler.token::Tok::Wild` | ctor | `lib/compiler/token.myc:114` | `Wild` | — | Empirical/Declared |
| `compiler.token::Tok::With` | ctor | `lib/compiler/token.myc:114` | `With` | — | Empirical/Declared |
| `compiler.token::Tok::Lambda` | ctor | `lib/compiler/token.myc:116` | `Lambda` | — | Empirical/Declared |
| `compiler.token::Tok::Lower` | ctor | `lib/compiler/token.myc:116` | `Lower` | — | Empirical/Declared |
| `compiler.token::Tok::Object` | ctor | `lib/compiler/token.myc:116` | `Object` | — | Empirical/Declared |
| `compiler.token::Tok::Via` | ctor | `lib/compiler/token.myc:116` | `Via` | — | Empirical/Declared |
| `compiler.token::Tok::Vsa` | ctor | `lib/compiler/token.myc:117` | `Vsa` | — | Empirical/Declared |
| `compiler.token::Tok::KwBinary` | ctor | `lib/compiler/token.myc:118` | `KwBinary` | — | Empirical/Declared |
| `compiler.token::Tok::KwDense` | ctor | `lib/compiler/token.myc:118` | `KwDense` | — | Empirical/Declared |
| `compiler.token::Tok::KwTernary` | ctor | `lib/compiler/token.myc:118` | `KwTernary` | — | Empirical/Declared |
| `compiler.token::Tok::BinShort` | ctor | `lib/compiler/token.myc:120` | `BinShort` | — | Empirical/Declared |
| `compiler.token::Tok::EmbShort` | ctor | `lib/compiler/token.myc:120` | `EmbShort` | — | Empirical/Declared |
| `compiler.token::Tok::HvecShort` | ctor | `lib/compiler/token.myc:120` | `HvecShort` | — | Empirical/Declared |
| `compiler.token::Tok::TernShort` | ctor | `lib/compiler/token.myc:120` | `TernShort` | — | Empirical/Declared |
| `compiler.token::Tok::KwBytes` | ctor | `lib/compiler/token.myc:121` | `KwBytes` | — | Empirical/Declared |
| `compiler.token::Tok::KwFloat` | ctor | `lib/compiler/token.myc:121` | `KwFloat` | — | Empirical/Declared |
| `compiler.token::Tok::KwSeq` | ctor | `lib/compiler/token.myc:121` | `KwSeq` | — | Empirical/Declared |
| `compiler.token::Tok::KwSparse` | ctor | `lib/compiler/token.myc:121` | `KwSparse` | — | Empirical/Declared |
| `compiler.token::Tok::KwSubstrate` | ctor | `lib/compiler/token.myc:121` | `KwSubstrate` | — | Empirical/Declared |
| `compiler.token::Tok::Scalar` | ctor | `lib/compiler/token.myc:123` | `Scalar(ScalarTok)` | — | Empirical/Declared |
| `compiler.token::Tok::Strength` | ctor | `lib/compiler/token.myc:123` | `Strength(StrengthTok)` | — | Empirical/Declared |
| `compiler.token::Tok::Ident` | ctor | `lib/compiler/token.myc:125` | `Ident(Bytes)` | — | Empirical/Declared |
| `compiler.token::Tok::BinLit` | ctor | `lib/compiler/token.myc:126` | `BinLit(Bytes)` | — | Empirical/Declared |
| `compiler.token::Tok::BytesLit` | ctor | `lib/compiler/token.myc:127` | `BytesLit(Bytes)` | — | Empirical/Declared |
| `compiler.token::Tok::TritLit` | ctor | `lib/compiler/token.myc:128` | `TritLit(Bytes)` | — | Empirical/Declared |
| `compiler.token::Tok::StrLit` | ctor | `lib/compiler/token.myc:129` | `StrLit(Bytes)` | — | Empirical/Declared |
| `compiler.token::Tok::Int` | ctor | `lib/compiler/token.myc:130` | `Int(Bytes)` | — | Empirical/Declared |
| `compiler.token::Tok::FloatLit` | ctor | `lib/compiler/token.myc:131` | `FloatLit(Bytes)` | — | Empirical/Declared |
| `compiler.token::Tok::LBrace` | ctor | `lib/compiler/token.myc:133` | `LBrace` | — | Empirical/Declared |
| `compiler.token::Tok::LBracket` | ctor | `lib/compiler/token.myc:133` | `LBracket` | — | Empirical/Declared |
| `compiler.token::Tok::LParen` | ctor | `lib/compiler/token.myc:133` | `LParen` | — | Empirical/Declared |
| `compiler.token::Tok::RBrace` | ctor | `lib/compiler/token.myc:133` | `RBrace` | — | Empirical/Declared |
| `compiler.token::Tok::RBracket` | ctor | `lib/compiler/token.myc:133` | `RBracket` | — | Empirical/Declared |
| `compiler.token::Tok::RParen` | ctor | `lib/compiler/token.myc:133` | `RParen` | — | Empirical/Declared |
| `compiler.token::Tok::LAngle` | ctor | `lib/compiler/token.myc:134` | `LAngle` | — | Empirical/Declared |
| `compiler.token::Tok::RAngle` | ctor | `lib/compiler/token.myc:134` | `RAngle` | — | Empirical/Declared |
| `compiler.token::Tok::Shl` | ctor | `lib/compiler/token.myc:134` | `Shl` | — | Empirical/Declared |
| `compiler.token::Tok::Shr` | ctor | `lib/compiler/token.myc:134` | `Shr` | — | Empirical/Declared |
| `compiler.token::Tok::At` | ctor | `lib/compiler/token.myc:135` | `At` | — | Empirical/Declared |
| `compiler.token::Tok::AtStdSys` | ctor | `lib/compiler/token.myc:135` | `AtStdSys` | — | Empirical/Declared |
| `compiler.token::Tok::Colon` | ctor | `lib/compiler/token.myc:136` | `Colon` | — | Empirical/Declared |
| `compiler.token::Tok::Comma` | ctor | `lib/compiler/token.myc:136` | `Comma` | — | Empirical/Declared |
| `compiler.token::Tok::Dot` | ctor | `lib/compiler/token.myc:136` | `Dot` | — | Empirical/Declared |
| `compiler.token::Tok::Pipe` | ctor | `lib/compiler/token.myc:136` | `Pipe` | — | Empirical/Declared |
| `compiler.token::Tok::Semi` | ctor | `lib/compiler/token.myc:136` | `Semi` | — | Empirical/Declared |
| `compiler.token::Tok::Amp` | ctor | `lib/compiler/token.myc:137` | `Amp` | — | Empirical/Declared |
| `compiler.token::Tok::AmpAmp` | ctor | `lib/compiler/token.myc:137` | `AmpAmp` | — | Empirical/Declared |
| `compiler.token::Tok::Caret` | ctor | `lib/compiler/token.myc:137` | `Caret` | — | Empirical/Declared |
| `compiler.token::Tok::Minus` | ctor | `lib/compiler/token.myc:137` | `Minus` | — | Empirical/Declared |
| `compiler.token::Tok::Percent` | ctor | `lib/compiler/token.myc:137` | `Percent` | — | Empirical/Declared |
| `compiler.token::Tok::Plus` | ctor | `lib/compiler/token.myc:137` | `Plus` | — | Empirical/Declared |
| `compiler.token::Tok::Slash` | ctor | `lib/compiler/token.myc:137` | `Slash` | — | Empirical/Declared |
| `compiler.token::Tok::Star` | ctor | `lib/compiler/token.myc:137` | `Star` | — | Empirical/Declared |
| `compiler.token::Tok::Arrow` | ctor | `lib/compiler/token.myc:138` | `Arrow` | — | Empirical/Declared |
| `compiler.token::Tok::Bang` | ctor | `lib/compiler/token.myc:138` | `Bang` | — | Empirical/Declared |
| `compiler.token::Tok::BangEq` | ctor | `lib/compiler/token.myc:138` | `BangEq` | — | Empirical/Declared |
| `compiler.token::Tok::Eq` | ctor | `lib/compiler/token.myc:138` | `Eq` | — | Empirical/Declared |
| `compiler.token::Tok::EqEq` | ctor | `lib/compiler/token.myc:138` | `EqEq` | — | Empirical/Declared |
| `compiler.token::Tok::FatArrow` | ctor | `lib/compiler/token.myc:138` | `FatArrow` | — | Empirical/Declared |
| `compiler.token::Tok::PipePipe` | ctor | `lib/compiler/token.myc:138` | `PipePipe` | — | Empirical/Declared |
| `compiler.token::Tok::Eof` | ctor | `lib/compiler/token.myc:139` | `Eof` | — | Empirical/Declared |
| `compiler.token::Spanned` | type | `lib/compiler/token.myc:143` | `type Spanned = Sp(Tok, Pos)` | — | Empirical/Declared |
| `compiler.token::Spanned::Sp` | ctor | `lib/compiler/token.myc:143` | `Sp(Tok, Pos)` | — | Empirical/Declared |
| `compiler.token::sp_tok` | fn | `lib/compiler/token.myc:145` | `fn sp_tok(s: Spanned) => Tok` | — | Empirical/Declared |
| `compiler.token::sp_pos` | fn | `lib/compiler/token.myc:148` | `fn sp_pos(s: Spanned) => Pos` | — | Empirical/Declared |
| `compiler.token::keyword` | fn | `lib/compiler/token.myc:157` | `fn keyword(word: Bytes) => Option[Tok]` | — | Empirical/Declared |

### compiler.totality

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `compiler.totality` | nodule | `lib/compiler/totality.myc:11` | `nodule compiler.totality` | Self-hosted structural totality checker (M-740 Stage 4; DN-26 §7.3 row 4). A faithful port of crates/mycelium-l1/src/totality.rs's `classify_all` (RFC-0007 §4.5 — Foetus-style structural descent, self- and mutual-recursion) over a SELF-CONTAINED copy of the `ast.myc` type vocabulary it traverses (`Expr`/`Pattern`/`Arm`/`FnDecl` and their transitive closure). | Empirical/Declared |
| `compiler.totality::Option` | type | `lib/compiler/totality.myc:100` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `compiler.totality::Option::None` | ctor | `lib/compiler/totality.myc:100` | `None` | — | Empirical/Declared |
| `compiler.totality::Option::Some` | ctor | `lib/compiler/totality.myc:100` | `Some(A)` | — | Empirical/Declared |
| `compiler.totality::Vec` | type | `lib/compiler/totality.myc:101` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.totality::Vec::Cons` | ctor | `lib/compiler/totality.myc:101` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `compiler.totality::Vec::Nil` | ctor | `lib/compiler/totality.myc:101` | `Nil` | — | Empirical/Declared |
| `compiler.totality::Result` | type | `lib/compiler/totality.myc:102` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `compiler.totality::Result::Err` | ctor | `lib/compiler/totality.myc:102` | `Err(E)` | — | Empirical/Declared |
| `compiler.totality::Result::Ok` | ctor | `lib/compiler/totality.myc:102` | `Ok(A)` | — | Empirical/Declared |
| `compiler.totality::Pair` | type | `lib/compiler/totality.myc:103` | `type Pair[A, B] = Pr(A, B)` | — | Empirical/Declared |
| `compiler.totality::Pair::Pr` | ctor | `lib/compiler/totality.myc:103` | `Pr(A, B)` | — | Empirical/Declared |
| `compiler.totality::pair_key` | fn | `lib/compiler/totality.myc:105` | `fn pair_key[A, B](p: Pair[A, B]) => A` | — | Empirical/Declared |
| `compiler.totality::pair_val` | fn | `lib/compiler/totality.myc:108` | `fn pair_val[A, B](p: Pair[A, B]) => B` | — | Empirical/Declared |
| `compiler.totality::zero32` | fn | `lib/compiler/totality.myc:112` | `fn zero32() => Binary{32}` | — | Empirical/Declared |
| `compiler.totality::one32` | fn | `lib/compiler/totality.myc:113` | `fn one32() => Binary{32}` | — | Empirical/Declared |
| `compiler.totality::Vis` | type | `lib/compiler/totality.myc:121` | `type Vis = Private \| Pub` | — | Empirical/Declared |
| `compiler.totality::Vis::Private` | ctor | `lib/compiler/totality.myc:121` | `Private` | — | Empirical/Declared |
| `compiler.totality::Vis::Pub` | ctor | `lib/compiler/totality.myc:121` | `Pub` | — | Empirical/Declared |
| `compiler.totality::Path` | type | `lib/compiler/totality.myc:124` | `type Path = Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.totality::Path::Pth` | ctor | `lib/compiler/totality.myc:124` | `Pth(Vec[Bytes])` | — | Empirical/Declared |
| `compiler.totality::path_segs` | fn | `lib/compiler/totality.myc:126` | `fn path_segs(p: Path) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.totality::Paradigm` | type | `lib/compiler/totality.myc:130` | `type Paradigm = PBinary \| PTernary \| PDense \| PVsa` | — | Empirical/Declared |
| `compiler.totality::Paradigm::PBinary` | ctor | `lib/compiler/totality.myc:130` | `PBinary` | — | Empirical/Declared |
| `compiler.totality::Paradigm::PDense` | ctor | `lib/compiler/totality.myc:130` | `PDense` | — | Empirical/Declared |
| `compiler.totality::Paradigm::PTernary` | ctor | `lib/compiler/totality.myc:130` | `PTernary` | — | Empirical/Declared |
| `compiler.totality::Paradigm::PVsa` | ctor | `lib/compiler/totality.myc:130` | `PVsa` | — | Empirical/Declared |
| `compiler.totality::Scalar` | type | `lib/compiler/totality.myc:133` | `type Scalar = SF16 \| SBf16 \| SF32 \| SF64` | — | Empirical/Declared |
| `compiler.totality::Scalar::SBf16` | ctor | `lib/compiler/totality.myc:133` | `SBf16` | — | Empirical/Declared |
| `compiler.totality::Scalar::SF16` | ctor | `lib/compiler/totality.myc:133` | `SF16` | — | Empirical/Declared |
| `compiler.totality::Scalar::SF32` | ctor | `lib/compiler/totality.myc:133` | `SF32` | — | Empirical/Declared |
| `compiler.totality::Scalar::SF64` | ctor | `lib/compiler/totality.myc:133` | `SF64` | — | Empirical/Declared |
| `compiler.totality::Sparsity` | type | `lib/compiler/totality.myc:136` | `type Sparsity = SpDense \| SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.totality::Sparsity::SpDense` | ctor | `lib/compiler/totality.myc:136` | `SpDense` | — | Empirical/Declared |
| `compiler.totality::Sparsity::SpSparse` | ctor | `lib/compiler/totality.myc:136` | `SpSparse(Binary{32})` | — | Empirical/Declared |
| `compiler.totality::AmbientParams` | type | `lib/compiler/totality.myc:139` | `type AmbientParams = APSize(Binary{32}) \| APDense(Binary{32}, Scalar) \| APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.totality::AmbientParams::APSize` | ctor | `lib/compiler/totality.myc:140` | `APSize(Binary{32})` | — | Empirical/Declared |
| `compiler.totality::AmbientParams::APDense` | ctor | `lib/compiler/totality.myc:141` | `APDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.totality::AmbientParams::APVsa` | ctor | `lib/compiler/totality.myc:142` | `APVsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.totality::Strength` | type | `lib/compiler/totality.myc:145` | `type Strength = GExact \| GProven \| GEmpirical \| GDeclared` | — | Empirical/Declared |
| `compiler.totality::Strength::GDeclared` | ctor | `lib/compiler/totality.myc:145` | `GDeclared` | — | Empirical/Declared |
| `compiler.totality::Strength::GEmpirical` | ctor | `lib/compiler/totality.myc:145` | `GEmpirical` | — | Empirical/Declared |
| `compiler.totality::Strength::GExact` | ctor | `lib/compiler/totality.myc:145` | `GExact` | — | Empirical/Declared |
| `compiler.totality::Strength::GProven` | ctor | `lib/compiler/totality.myc:145` | `GProven` | — | Empirical/Declared |
| `compiler.totality::WidthRef` | type | `lib/compiler/totality.myc:148` | `type WidthRef = WLit(Binary{32}) \| WName(Bytes)` | — | Empirical/Declared |
| `compiler.totality::WidthRef::WLit` | ctor | `lib/compiler/totality.myc:148` | `WLit(Binary{32})` | — | Empirical/Declared |
| `compiler.totality::WidthRef::WName` | ctor | `lib/compiler/totality.myc:148` | `WName(Bytes)` | — | Empirical/Declared |
| `compiler.totality::ParamKind` | type | `lib/compiler/totality.myc:151` | `type ParamKind = PkType \| PkWidth` | — | Empirical/Declared |
| `compiler.totality::ParamKind::PkType` | ctor | `lib/compiler/totality.myc:151` | `PkType` | — | Empirical/Declared |
| `compiler.totality::ParamKind::PkWidth` | ctor | `lib/compiler/totality.myc:151` | `PkWidth` | — | Empirical/Declared |
| `compiler.totality::TraitRef` | type | `lib/compiler/totality.myc:154` | `type TraitRef = TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.totality::TraitRef::TRf` | ctor | `lib/compiler/totality.myc:154` | `TRf(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.totality::TypeParam` | type | `lib/compiler/totality.myc:157` | `type TypeParam = TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.totality::TypeParam::TP` | ctor | `lib/compiler/totality.myc:157` | `TP(Bytes, ParamKind, Vec[TraitRef])` | — | Empirical/Declared |
| `compiler.totality::EffectBudget` | type | `lib/compiler/totality.myc:160` | `type EffectBudget = EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.totality::EffectBudget::EB` | ctor | `lib/compiler/totality.myc:160` | `EB(Bytes, Binary{64})` | — | Empirical/Declared |
| `compiler.totality::FnSig` | type | `lib/compiler/totality.myc:163` | `type FnSig = FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.totality::FnSig::FS` | ctor | `lib/compiler/totality.myc:163` | `FS(Bytes, Vec[TypeParam], Vec[Param], TypeRef, Vec[Bytes], Vec[EffectBudget])` | — | Empirical/Declared |
| `compiler.totality::fnsig_name` | fn | `lib/compiler/totality.myc:165` | `fn fnsig_name(s: FnSig) => Bytes` | — | Empirical/Declared |
| `compiler.totality::fnsig_value_params` | fn | `lib/compiler/totality.myc:168` | `fn fnsig_value_params(s: FnSig) => Vec[Param]` | — | Empirical/Declared |
| `compiler.totality::Param` | type | `lib/compiler/totality.myc:172` | `type Param = Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.totality::Param::Prm` | ctor | `lib/compiler/totality.myc:172` | `Prm(Bytes, TypeRef)` | — | Empirical/Declared |
| `compiler.totality::param_name` | fn | `lib/compiler/totality.myc:174` | `fn param_name(p: Param) => Bytes` | — | Empirical/Declared |
| `compiler.totality::TypeRef` | type | `lib/compiler/totality.myc:180` | `type TypeRef = TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.totality::TypeRef::TR` | ctor | `lib/compiler/totality.myc:180` | `TR(BaseType, Option[Strength])` | — | Empirical/Declared |
| `compiler.totality::BaseType` | type | `lib/compiler/totality.myc:182` | `type BaseType = KwBinary(WidthRef) \| KwTernary(WidthRef) \| KwDense(Binary{32}, Scalar) \| Vsa(Bytes, Binary{32}, Sparsity) \| KwSubstrate(Bytes) \| KwSeq(TypeRef, Binary{32}) \| KwBytes \| KwFloat \| Named(Bytes, Vec[TypeRef]) \| Ambient(AmbientParams) \| FnArrow(TypeRef, TypeRef) \| Tuple(Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.totality::BaseType::KwBinary` | ctor | `lib/compiler/totality.myc:183` | `KwBinary(WidthRef)` | — | Empirical/Declared |
| `compiler.totality::BaseType::KwTernary` | ctor | `lib/compiler/totality.myc:184` | `KwTernary(WidthRef)` | — | Empirical/Declared |
| `compiler.totality::BaseType::KwDense` | ctor | `lib/compiler/totality.myc:185` | `KwDense(Binary{32}, Scalar)` | — | Empirical/Declared |
| `compiler.totality::BaseType::Vsa` | ctor | `lib/compiler/totality.myc:186` | `Vsa(Bytes, Binary{32}, Sparsity)` | — | Empirical/Declared |
| `compiler.totality::BaseType::KwSubstrate` | ctor | `lib/compiler/totality.myc:187` | `KwSubstrate(Bytes)` | — | Empirical/Declared |
| `compiler.totality::BaseType::KwSeq` | ctor | `lib/compiler/totality.myc:188` | `KwSeq(TypeRef, Binary{32})` | — | Empirical/Declared |
| `compiler.totality::BaseType::KwBytes` | ctor | `lib/compiler/totality.myc:189` | `KwBytes` | — | Empirical/Declared |
| `compiler.totality::BaseType::KwFloat` | ctor | `lib/compiler/totality.myc:190` | `KwFloat` | — | Empirical/Declared |
| `compiler.totality::BaseType::Named` | ctor | `lib/compiler/totality.myc:191` | `Named(Bytes, Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.totality::BaseType::Ambient` | ctor | `lib/compiler/totality.myc:192` | `Ambient(AmbientParams)` | — | Empirical/Declared |
| `compiler.totality::BaseType::FnArrow` | ctor | `lib/compiler/totality.myc:193` | `FnArrow(TypeRef, TypeRef)` | — | Empirical/Declared |
| `compiler.totality::BaseType::Tuple` | ctor | `lib/compiler/totality.myc:194` | `Tuple(Vec[TypeRef])` | — | Empirical/Declared |
| `compiler.totality::ExecutionMode` | type | `lib/compiler/totality.myc:197` | `type ExecutionMode = Interpreted \| Compiled` | — | Empirical/Declared |
| `compiler.totality::ExecutionMode::Compiled` | ctor | `lib/compiler/totality.myc:197` | `Compiled` | — | Empirical/Declared |
| `compiler.totality::ExecutionMode::Interpreted` | ctor | `lib/compiler/totality.myc:197` | `Interpreted` | — | Empirical/Declared |
| `compiler.totality::FnDecl` | type | `lib/compiler/totality.myc:200` | `type FnDecl = FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.totality::FnDecl::FD` | ctor | `lib/compiler/totality.myc:200` | `FD(Vis, Bool, Option[ExecutionMode], FnSig, Expr)` | — | Empirical/Declared |
| `compiler.totality::fndecl_sig` | fn | `lib/compiler/totality.myc:202` | `fn fndecl_sig(f: FnDecl) => FnSig` | — | Empirical/Declared |
| `compiler.totality::fndecl_body` | fn | `lib/compiler/totality.myc:205` | `fn fndecl_body(f: FnDecl) => Expr` | — | Empirical/Declared |
| `compiler.totality::Literal` | type | `lib/compiler/totality.myc:210` | `type Literal = Bin(Bytes) \| Trit(Bytes) \| Int(Binary{64}) \| AmbientInt(Paradigm, Binary{64}) \| List(Vec[Expr]) \| LBytes(Bytes) \| Str(Bytes) \| LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.totality::Literal::Bin` | ctor | `lib/compiler/totality.myc:211` | `Bin(Bytes)` | — | Empirical/Declared |
| `compiler.totality::Literal::Trit` | ctor | `lib/compiler/totality.myc:212` | `Trit(Bytes)` | — | Empirical/Declared |
| `compiler.totality::Literal::Int` | ctor | `lib/compiler/totality.myc:213` | `Int(Binary{64})` | — | Empirical/Declared |
| `compiler.totality::Literal::AmbientInt` | ctor | `lib/compiler/totality.myc:214` | `AmbientInt(Paradigm, Binary{64})` | — | Empirical/Declared |
| `compiler.totality::Literal::List` | ctor | `lib/compiler/totality.myc:215` | `List(Vec[Expr])` | — | Empirical/Declared |
| `compiler.totality::Literal::LBytes` | ctor | `lib/compiler/totality.myc:216` | `LBytes(Bytes)` | — | Empirical/Declared |
| `compiler.totality::Literal::Str` | ctor | `lib/compiler/totality.myc:217` | `Str(Bytes)` | — | Empirical/Declared |
| `compiler.totality::Literal::LFloat` | ctor | `lib/compiler/totality.myc:218` | `LFloat(Bytes)` | — | Empirical/Declared |
| `compiler.totality::Pattern` | type | `lib/compiler/totality.myc:221` | `type Pattern = PWildcard \| PLit(Literal) \| PCtor(Bytes, Vec[Pattern]) \| PIdent(Bytes) \| PTuple(Vec[Pattern]) \| POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.totality::Pattern::PWildcard` | ctor | `lib/compiler/totality.myc:222` | `PWildcard` | — | Empirical/Declared |
| `compiler.totality::Pattern::PLit` | ctor | `lib/compiler/totality.myc:223` | `PLit(Literal)` | — | Empirical/Declared |
| `compiler.totality::Pattern::PCtor` | ctor | `lib/compiler/totality.myc:224` | `PCtor(Bytes, Vec[Pattern])` | — | Empirical/Declared |
| `compiler.totality::Pattern::PIdent` | ctor | `lib/compiler/totality.myc:225` | `PIdent(Bytes)` | — | Empirical/Declared |
| `compiler.totality::Pattern::PTuple` | ctor | `lib/compiler/totality.myc:226` | `PTuple(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.totality::Pattern::POr` | ctor | `lib/compiler/totality.myc:227` | `POr(Vec[Pattern])` | — | Empirical/Declared |
| `compiler.totality::Arm` | type | `lib/compiler/totality.myc:230` | `type Arm = Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.totality::Arm::Ar` | ctor | `lib/compiler/totality.myc:230` | `Ar(Pattern, Expr)` | — | Empirical/Declared |
| `compiler.totality::arm_pattern` | fn | `lib/compiler/totality.myc:232` | `fn arm_pattern(a: Arm) => Pattern` | — | Empirical/Declared |
| `compiler.totality::arm_body` | fn | `lib/compiler/totality.myc:235` | `fn arm_body(a: Arm) => Expr` | — | Empirical/Declared |
| `compiler.totality::Hypha` | type | `lib/compiler/totality.myc:239` | `type Hypha = Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.totality::Hypha::Hy` | ctor | `lib/compiler/totality.myc:239` | `Hy(Option[Expr], Expr)` | — | Empirical/Declared |
| `compiler.totality::hypha_body` | fn | `lib/compiler/totality.myc:241` | `fn hypha_body(h: Hypha) => Expr` | — | Empirical/Declared |
| `compiler.totality::Expr` | type | `lib/compiler/totality.myc:245` | `type Expr = Let(Bytes, Option[TypeRef], Expr, Expr) \| If(Expr, Expr, Expr) \| Match(Expr, Vec[Arm]) \| For(Bytes, Expr, Bytes, Expr, Expr) \| Swap(Expr, TypeRef, Path) \| WithParadigm(Paradigm, Expr) \| Wild(Expr) \| Spore(Expr) \| Consume(Expr) \| Colony(Vec[Hypha]) \| Lambda(Vec[Param], Expr) \| App(Expr, Vec[Expr]) \| Fuse(Expr, Expr) \| Reclaim(Expr, Expr) \| Path(Path) \| Lit(Literal) \| Ascribe(Expr, TypeRef) \| TupleLit(Vec[Expr])` | — | Empirical/Declared |
| `compiler.totality::Expr::Let` | ctor | `lib/compiler/totality.myc:246` | `Let(Bytes, Option[TypeRef], Expr, Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::If` | ctor | `lib/compiler/totality.myc:247` | `If(Expr, Expr, Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::Match` | ctor | `lib/compiler/totality.myc:248` | `Match(Expr, Vec[Arm])` | — | Empirical/Declared |
| `compiler.totality::Expr::For` | ctor | `lib/compiler/totality.myc:249` | `For(Bytes, Expr, Bytes, Expr, Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::Path` | ctor | `lib/compiler/totality.myc:250` | `Path(Path)` | — | Empirical/Declared |
| `compiler.totality::Expr::Swap` | ctor | `lib/compiler/totality.myc:250` | `Swap(Expr, TypeRef, Path)` | — | Empirical/Declared |
| `compiler.totality::Expr::WithParadigm` | ctor | `lib/compiler/totality.myc:251` | `WithParadigm(Paradigm, Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::Wild` | ctor | `lib/compiler/totality.myc:252` | `Wild(Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::Spore` | ctor | `lib/compiler/totality.myc:253` | `Spore(Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::Consume` | ctor | `lib/compiler/totality.myc:254` | `Consume(Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::Colony` | ctor | `lib/compiler/totality.myc:255` | `Colony(Vec[Hypha])` | — | Empirical/Declared |
| `compiler.totality::Expr::Lambda` | ctor | `lib/compiler/totality.myc:256` | `Lambda(Vec[Param], Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::App` | ctor | `lib/compiler/totality.myc:257` | `App(Expr, Vec[Expr])` | — | Empirical/Declared |
| `compiler.totality::Expr::Fuse` | ctor | `lib/compiler/totality.myc:258` | `Fuse(Expr, Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::Reclaim` | ctor | `lib/compiler/totality.myc:259` | `Reclaim(Expr, Expr)` | — | Empirical/Declared |
| `compiler.totality::Expr::Lit` | ctor | `lib/compiler/totality.myc:261` | `Lit(Literal)` | — | Empirical/Declared |
| `compiler.totality::Expr::Ascribe` | ctor | `lib/compiler/totality.myc:262` | `Ascribe(Expr, TypeRef)` | — | Empirical/Declared |
| `compiler.totality::Expr::TupleLit` | ctor | `lib/compiler/totality.myc:263` | `TupleLit(Vec[Expr])` | — | Empirical/Declared |
| `compiler.totality::Totality` | type | `lib/compiler/totality.myc:266` | `type Totality = Total \| Partial` | — | Empirical/Declared |
| `compiler.totality::Totality::Partial` | ctor | `lib/compiler/totality.myc:266` | `Partial` | — | Empirical/Declared |
| `compiler.totality::Totality::Total` | ctor | `lib/compiler/totality.myc:266` | `Total` | — | Empirical/Declared |
| `compiler.totality::WalkDepthExceeded` | type | `lib/compiler/totality.myc:269` | `type WalkDepthExceeded = WDE(Binary{32})` | — | Empirical/Declared |
| `compiler.totality::WalkDepthExceeded::WDE` | ctor | `lib/compiler/totality.myc:269` | `WDE(Binary{32})` | — | Empirical/Declared |
| `compiler.totality::max_walk_depth` | fn | `lib/compiler/totality.myc:273` | `fn max_walk_depth() => Binary{32}` | max_walk_depth: mirrors totality.rs::MAX_WALK_DEPTH (4096) exactly — the same numeric ceiling parse.myc's own `max_expr_depth` uses (FLAG-parse-7's sibling budget, a DIFFERENT traversal). | Empirical/Declared |
| `compiler.totality::max_assignments` | fn | `lib/compiler/totality.myc:278` | `fn max_assignments() => Binary{32}` | max_assignments: mirrors totality.rs::MAX_ASSIGNMENTS (4096) — a DIFFERENT budget (a bound on the mutual-descent position-assignment SEARCH SPACE, not on AST nesting) that happens to share the same numeric value; kept as its own named constant for honesty (never conflated in-file). | Empirical/Declared |
| `compiler.totality::charge_depth` | fn | `lib/compiler/totality.myc:283` | `fn charge_depth(depth: Binary{32}) => Result[Binary{32}, WalkDepthExceeded]` | charge_depth: mirrors the `let depth = depth + 1; if depth > MAX_WALK_DEPTH { return Err(...) }` prologue shared by totality.rs's `walk_expr_at`/`descend_walk`/`pattern_binders` (M-674's FLAG-totality-3 threaded-budget discipline). | Empirical/Declared |
| `compiler.totality::alist_get` | fn | `lib/compiler/totality.myc:293` | `fn alist_get[V](xs: Vec[Pair[Bytes, V]], k: Bytes) => Option[V]` | alist_get: linear-scan lookup by Bytes key (the BTreeMap::get / BTreeMap::contains_key mirror). | Empirical/Declared |
| `compiler.totality::contains_bytes` | fn | `lib/compiler/totality.myc:304` | `fn contains_bytes(xs: Vec[Bytes], x: Bytes) => Bool` | contains_bytes / add_bytes_if_absent: the parse.myc convention (redeclared, self-contained) — the BTreeSet<String>::contains / ::insert mirror over a plain Vec[Bytes]. | Empirical/Declared |
| `compiler.totality::add_bytes_if_absent` | fn | `lib/compiler/totality.myc:313` | `fn add_bytes_if_absent(xs: Vec[Bytes], x: Bytes) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.totality::remove_bytes_acc` | fn | `lib/compiler/totality.myc:321` | `fn remove_bytes_acc(xs: Vec[Bytes], x: Bytes, acc: Vec[Bytes]) => Vec[Bytes]` | remove_bytes: drop every occurrence of `x` from `xs` (the BTreeSet::remove mirror; direct-tail accumulator + rev_acc, bounded by list LENGTH — idiom #2). | Empirical/Declared |
| `compiler.totality::remove_bytes` | fn | `lib/compiler/totality.myc:330` | `fn remove_bytes(xs: Vec[Bytes], x: Bytes) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.totality::remove_all_bytes` | fn | `lib/compiler/totality.myc:333` | `fn remove_all_bytes(xs: Vec[Bytes], names: Vec[Bytes]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.totality::add_all_bytes` | fn | `lib/compiler/totality.myc:339` | `fn add_all_bytes(xs: Vec[Bytes], names: Vec[Bytes]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.totality::append_bytes_acc` | fn | `lib/compiler/totality.myc:347` | `fn append_bytes_acc(xs: Vec[Bytes], ys: Vec[Bytes], acc: Vec[Bytes]) => Vec[Bytes]` | append_bytes: `xs ++ ys` (order-irrelevant for the reachability worklist it feeds — direct-tail accumulator + rev_acc, bounded by `xs`'s length — idiom #2). | Empirical/Declared |
| `compiler.totality::append_bytes` | fn | `lib/compiler/totality.myc:353` | `fn append_bytes(xs: Vec[Bytes], ys: Vec[Bytes]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.totality::rev_acc` | fn | `lib/compiler/totality.myc:358` | `fn rev_acc[A](xs: Vec[A], acc: Vec[A]) => Vec[A]` | rev_acc: the direct-tail list reversal underpinning every accumulator+reverse builder above (the parse.myc M-985 convention, redeclared per self-containment). | Empirical/Declared |
| `compiler.totality::vec_param_len` | fn | `lib/compiler/totality.myc:363` | `fn vec_param_len(ps: Vec[Param]) => Binary{32}` | — | Empirical/Declared |
| `compiler.totality::vec_param_get` | fn | `lib/compiler/totality.myc:366` | `fn vec_param_get(xs: Vec[Param], i: Binary{32}) => Option[Param]` | — | Empirical/Declared |
| `compiler.totality::vec_expr_get` | fn | `lib/compiler/totality.myc:375` | `fn vec_expr_get(xs: Vec[Expr], i: Binary{32}) => Option[Expr]` | — | Empirical/Declared |
| `compiler.totality::app_call_name` | fn | `lib/compiler/totality.myc:390` | `fn app_call_name(head: Expr) => Option[Bytes]` | app_call_name: if `head` is a single-segment `Path`, its name; otherwise `None`. Mirrors the `if let Expr::Path(p) = head.as_ref() { if p.0.len() == 1 { ... } }` guard shared by `collect_calls`'s visitor and `descend_walk`'s `App` case. | Empirical/Declared |
| `compiler.totality::expr_is_smaller` | fn | `lib/compiler/totality.myc:404` | `fn expr_is_smaller(e: Expr, param: Bytes, smaller: Vec[Bytes]) => Bool` | expr_is_smaller: mirrors `match scrutinee.as_ref() { Expr::Path(p) if p.0.len()==1 => p.0[0] == param \|\| smaller.contains(&p.0[0]), _ => false }`. | Empirical/Declared |
| `compiler.totality::collect_calls_expr` | fn | `lib/compiler/totality.myc:423` | `fn collect_calls_expr(e: Expr, fn_names: Vec[Bytes], depth: Binary{32}, acc: Vec[Bytes]) => Result[Vec[Bytes], WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::collect_calls_list` | fn | `lib/compiler/totality.myc:493` | `fn collect_calls_list(xs: Vec[Expr], fn_names: Vec[Bytes], depth: Binary{32}, acc: Vec[Bytes]) => Result[Vec[Bytes], WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::collect_calls_arm_list` | fn | `lib/compiler/totality.myc:502` | `fn collect_calls_arm_list(xs: Vec[Arm], fn_names: Vec[Bytes], depth: Binary{32}, acc: Vec[Bytes]) => Result[Vec[Bytes], WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::collect_calls_hypha_list` | fn | `lib/compiler/totality.myc:511` | `fn collect_calls_hypha_list(xs: Vec[Hypha], fn_names: Vec[Bytes], depth: Binary{32}, acc: Vec[Bytes]) => Result[Vec[Bytes], WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::fn_table_names_acc` | fn | `lib/compiler/totality.myc:525` | `fn fn_table_names_acc(fns: Vec[Pair[Bytes, FnDecl]], acc: Vec[Bytes]) => Vec[Bytes]` | fn_table_names: extract just the names, ORDER PRESERVED (the caller's sortedness precondition, FLAG-totality-1, rides through unchanged). Direct-tail accumulator + rev_acc — bounded by the number of functions in the table (idiom #2). | Empirical/Declared |
| `compiler.totality::fn_table_names` | fn | `lib/compiler/totality.myc:531` | `fn fn_table_names(fns: Vec[Pair[Bytes, FnDecl]]) => Vec[Bytes]` | — | Empirical/Declared |
| `compiler.totality::build_calls` | fn | `lib/compiler/totality.myc:536` | `fn build_calls(fns: Vec[Pair[Bytes, FnDecl]], fn_names: Vec[Bytes], acc: Vec[Pair[Bytes, Vec[Bytes]]]) => Result[Vec[Pair[Bytes, Vec[Bytes]]], WalkDepthExceeded]` | build_calls: one collect_calls_expr per function, in table order (order irrelevant to the resulting lookup-only map — FLAG-totality-1). Direct-tail. | Empirical/Declared |
| `compiler.totality::reaches_step` | fn | `lib/compiler/totality.myc:548` | `fn reaches_step(target: Bytes, calls: Vec[Pair[Bytes, Vec[Bytes]]], seen: Vec[Bytes], stack: Vec[Bytes]) => Bool` | — | Empirical/Declared |
| `compiler.totality::reaches` | fn | `lib/compiler/totality.myc:565` | `fn reaches(from: Bytes, target: Bytes, calls: Vec[Pair[Bytes, Vec[Bytes]]]) => Bool` | — | Empirical/Declared |
| `compiler.totality::scc_inner_scan` | fn | `lib/compiler/totality.myc:571` | `fn scc_inner_scan(others: Vec[Bytes], calls: Vec[Pair[Bytes, Vec[Bytes]]], name: Bytes, assigned: Vec[Bytes], group: Vec[Bytes]) => Pair[Vec[Bytes], Vec[Bytes]]` | — | Empirical/Declared |
| `compiler.totality::scc_outer` | fn | `lib/compiler/totality.myc:586` | `fn scc_outer(remaining: Vec[Bytes], fn_names: Vec[Bytes], calls: Vec[Pair[Bytes, Vec[Bytes]]], assigned: Vec[Bytes], acc: Vec[Vec[Bytes]]) => Vec[Vec[Bytes]]` | — | Empirical/Declared |
| `compiler.totality::strongly_connected` | fn | `lib/compiler/totality.myc:599` | `fn strongly_connected(fn_names: Vec[Bytes], calls: Vec[Pair[Bytes, Vec[Bytes]]]) => Vec[Vec[Bytes]]` | — | Empirical/Declared |
| `compiler.totality::scc_members` | fn | `lib/compiler/totality.myc:604` | `fn scc_members(scc: Vec[Bytes], fns: Vec[Pair[Bytes, FnDecl]]) => Vec[FnDecl]` | — | Empirical/Declared |
| `compiler.totality::member_arities` | fn | `lib/compiler/totality.myc:617` | `fn member_arities(members: Vec[FnDecl]) => Vec[Binary{32}]` | — | Empirical/Declared |
| `compiler.totality::any_zero` | fn | `lib/compiler/totality.myc:623` | `fn any_zero(xs: Vec[Binary{32}]) => Bool` | — | Empirical/Declared |
| `compiler.totality::arity_product_capped_acc` | fn | `lib/compiler/totality.myc:631` | `fn arity_product_capped_acc(arities: Vec[Binary{32}], running: Binary{32}, cap: Binary{32}) => Option[Binary{32}]` | arity_product_capped: FLAG-totality-5's early-exit product (equivalent to, cheaper than, the oracle's "compute in full, compare after"). | Empirical/Declared |
| `compiler.totality::arity_product_capped` | fn | `lib/compiler/totality.myc:642` | `fn arity_product_capped(arities: Vec[Binary{32}], cap: Binary{32}) => Option[Binary{32}]` | — | Empirical/Declared |
| `compiler.totality::decode_pos_acc` | fn | `lib/compiler/totality.myc:648` | `fn decode_pos_acc(members: Vec[FnDecl], arities: Vec[Binary{32}], rem: Binary{32}, acc: Vec[Pair[Bytes, Binary{32}]]) => Vec[Pair[Bytes, Binary{32}]]` | decode_pos: the mixed-radix digit decomposition of `rem` over `arities`, zipped with `members`' names (mirrors group_descends's `pos.insert(fd.sig.name.as_str(), rem % arity); rem /= arity`). Order among `pos` entries is irrelevant (assoc list, lookup only — FLAG-totality-1). | Empirical/Declared |
| `compiler.totality::decode_pos` | fn | `lib/compiler/totality.myc:663` | `fn decode_pos(members: Vec[FnDecl], arities: Vec[Binary{32}], rem: Binary{32}) => Vec[Pair[Bytes, Binary{32}]]` | — | Empirical/Declared |
| `compiler.totality::app_call_ok` | fn | `lib/compiler/totality.myc:668` | `fn app_call_ok(head: Expr, args: Vec[Expr], pos: Vec[Pair[Bytes, Binary{32}]], smaller: Vec[Bytes], ok: Bool) => Bool` | — | Empirical/Declared |
| `compiler.totality::descend_walk` | fn | `lib/compiler/totality.myc:701` | `fn descend_walk(e: Expr, pos: Vec[Pair[Bytes, Binary{32}]], param: Bytes, smaller: Vec[Bytes], ok: Bool, depth: Binary{32}) => Result[Bool, WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::descend_walk_list` | fn | `lib/compiler/totality.myc:769` | `fn descend_walk_list(xs: Vec[Expr], pos: Vec[Pair[Bytes, Binary{32}]], param: Bytes, smaller: Vec[Bytes], ok: Bool, depth: Binary{32}) => Result[Bool, WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::descend_walk_hyphae` | fn | `lib/compiler/totality.myc:778` | `fn descend_walk_hyphae(xs: Vec[Hypha], pos: Vec[Pair[Bytes, Binary{32}]], param: Bytes, smaller: Vec[Bytes], ok: Bool, depth: Binary{32}) => Result[Bool, WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::descend_walk_arms` | fn | `lib/compiler/totality.myc:789` | `fn descend_walk_arms(arms: Vec[Arm], pos: Vec[Pair[Bytes, Binary{32}]], param: Bytes, smaller_outer: Vec[Bytes], ok: Bool, scrut_small: Bool, depth: Binary{32}) => Result[Bool, WalkDepthExceeded]` | descend_walk_arms: mirrors the Match-arm shadow/promote-then-restore dance (see the file-level note above for why no explicit "restore" step is needed in this functional threading). | Empirical/Declared |
| `compiler.totality::pattern_binders_list` | fn | `lib/compiler/totality.myc:814` | `fn pattern_binders_list(xs: Vec[Pattern], depth: Binary{32}, acc: Vec[Bytes]) => Result[Vec[Bytes], WalkDepthExceeded]` | pattern_binders: mirrors totality.rs::pattern_binders (every variable a pattern binds, recursively). FLAG-totality-4 covers the `POr` dead-fallback. | Empirical/Declared |
| `compiler.totality::pattern_binders_acc` | fn | `lib/compiler/totality.myc:823` | `fn pattern_binders_acc(p: Pattern, depth: Binary{32}, acc: Vec[Bytes]) => Result[Vec[Bytes], WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::pattern_binders` | fn | `lib/compiler/totality.myc:837` | `fn pattern_binders(p: Pattern, depth: Binary{32}) => Result[Vec[Bytes], WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::assignment_descends_scan` | fn | `lib/compiler/totality.myc:842` | `fn assignment_descends_scan(members: Vec[FnDecl], pos: Vec[Pair[Bytes, Binary{32}]]) => Result[Bool, WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::assignment_descends` | fn | `lib/compiler/totality.myc:868` | `fn assignment_descends(members: Vec[FnDecl], pos: Vec[Pair[Bytes, Binary{32}]]) => Result[Bool, WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::group_descends_search` | fn | `lib/compiler/totality.myc:871` | `fn group_descends_search(members: Vec[FnDecl], arities: Vec[Binary{32}], rem: Binary{32}, combos: Binary{32}) => Result[Bool, WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::group_descends` | fn | `lib/compiler/totality.myc:885` | `fn group_descends(scc: Vec[Bytes], fns: Vec[Pair[Bytes, FnDecl]]) => Result[Bool, WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::scc_recursive_flag` | fn | `lib/compiler/totality.myc:899` | `fn scc_recursive_flag(scc: Vec[Bytes], calls: Vec[Pair[Bytes, Vec[Bytes]]]) => Bool` | — | Empirical/Declared |
| `compiler.totality::assign_totality_all` | fn | `lib/compiler/totality.myc:910` | `fn assign_totality_all(scc: Vec[Bytes], t: Totality, acc: Vec[Pair[Bytes, Totality]]) => Vec[Pair[Bytes, Totality]]` | — | Empirical/Declared |
| `compiler.totality::classify_sccs` | fn | `lib/compiler/totality.myc:916` | `fn classify_sccs(sccs: Vec[Vec[Bytes]], fns: Vec[Pair[Bytes, FnDecl]], calls: Vec[Pair[Bytes, Vec[Bytes]]], acc: Vec[Pair[Bytes, Totality]]) => Result[Vec[Pair[Bytes, Totality]], WalkDepthExceeded]` | — | Empirical/Declared |
| `compiler.totality::classify_all` | fn | `lib/compiler/totality.myc:932` | `fn classify_all(fns: Vec[Pair[Bytes, FnDecl]]) => Result[Vec[Pair[Bytes, Totality]], WalkDepthExceeded]` | classify_all: the public entry (mirrors totality.rs::classify_all/classify_all_inner). PRECONDITION (FLAG-totality-1, not enforced by this file): `fns` must already be sorted ascending by name. | Empirical/Declared |

## std

### std.cmp

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.cmp` | nodule | `lib/std/cmp.myc:8` | `nodule std.cmp` | Self-hosted ordering/equality surface — the Ordering type (Lt \| Eq \| Gt), its | Empirical/Declared |
| `std.cmp::Ordering` | type | `lib/std/cmp.myc:20` | `type Ordering = Lt \| Eq \| Gt` | projections (is_lt/is_eq/is_gt, reverse), and the structural equality/total-order functions over finite kernel types (Bool, Ordering), plus WIDTH-GENERIC comparison helpers over Binary{N} (M-715 E13-1 / M-718, RFC-0031 §5 D4 Tier-0/Tier-1; DN-42 width-generics). Guarantee: every op is total over its domain and match-defined or prim-backed; differential agreement (L1-eval ≡ L0-interp ≡ AOT) is Empirical (trials, std_cmp.rs). The width-generic helpers (cmp/le/ge/max/min) wrap the `eq`/`lt` kernel prims which are Exact over the finite Binary{N} domain — tagged Exact for every concrete width the call site pins. Never-silent (G2): no op fabricates an ordering for a value it cannot compare — the surface only covers types it can decide structurally or via the surfaced prims; an undetermined width is an explicit monomorphization refusal, never a guessed default. The three-valued total-order result. Total and finite: every comparison lands in exactly one arm. | Empirical/Declared |
| `std.cmp::Ordering::Eq` | ctor | `lib/std/cmp.myc:20` | `Eq` | — | Empirical/Declared |
| `std.cmp::Ordering::Gt` | ctor | `lib/std/cmp.myc:20` | `Gt` | — | Empirical/Declared |
| `std.cmp::Ordering::Lt` | ctor | `lib/std/cmp.myc:20` | `Lt` | — | Empirical/Declared |
| `std.cmp::is_lt` | fn | `lib/std/cmp.myc:23` | `fn is_lt(o: Ordering) => Bool` | is_lt / is_eq / is_gt: project an Ordering to Bool. Total over the finite Ordering domain (Exact). | Empirical/Declared |
| `std.cmp::is_eq` | fn | `lib/std/cmp.myc:26` | `fn is_eq(o: Ordering) => Bool` | — | Empirical/Declared |
| `std.cmp::is_gt` | fn | `lib/std/cmp.myc:29` | `fn is_gt(o: Ordering) => Bool` | — | Empirical/Declared |
| `std.cmp::reverse` | fn | `lib/std/cmp.myc:34` | `fn reverse(o: Ordering) => Ordering` | reverse: swap Lt <-> Gt, fix Eq — the order-reversal that turns an ascending comparator into a descending one. An involution; total over Ordering (Exact). | Empirical/Declared |
| `std.cmp::bool_eq` | fn | `lib/std/cmp.myc:38` | `fn bool_eq(a: Bool, b: Bool) => Bool` | bool_eq: structural equality on Bool, match-defined (no kernel eq prim needed). Total (Exact). | Empirical/Declared |
| `std.cmp::bool_cmp` | fn | `lib/std/cmp.myc:45` | `fn bool_cmp(a: Bool, b: Bool) => Ordering` | bool_cmp: the canonical total order on Bool with False < True. Total over Bool x Bool (Exact). | Empirical/Declared |
| `std.cmp::ord_eq` | fn | `lib/std/cmp.myc:53` | `fn ord_eq(a: Ordering, b: Ordering) => Bool` | ord_eq: structural equality on Ordering itself (a finite enum). Total (Exact). Useful for asserting a comparator's result without re-matching at every call site. | Empirical/Declared |
| `std.cmp::cmp` | fn | `lib/std/cmp.myc:85` | `fn cmp{N}(a: Binary{N}, b: Binary{N}) => Ordering` | DRY: `le/ge/max/min` are defined as projections of `cmp` — width-generic-to-width-generic delegation (one width-generic fn calling another with a still-ABSTRACT width) is supported as of M-718: the `unify` width-var pass-through binds the callee's width var to the caller's, mirroring the type-var pass-through, so the callee's width is "determined by the enclosing scope" and resolved at monomorphization. (Before M-718 this delegation was refused, so the wave-n1 `_u8` helpers inlined `eq`/`lt`; that limitation is now CLOSED — the recursive width-generic `map_get`/`set_contains` ride the same pass-through.) cmp<N>: three-way comparison of two Binary{N} values (unsigned magnitude). Eq if equal, Lt if a < b, Gt otherwise. Total over Binary{N} x Binary{N} for every concrete N (Exact: eq/lt are Exact prims; every case is covered by the nested match). | Empirical/Declared |
| `std.cmp::le` | fn | `lib/std/cmp.myc:89` | `fn le{N}(a: Binary{N}, b: Binary{N}) => Bool` | le<N>: True iff a <= b — a projection of cmp (Exact; total over Binary{N} x Binary{N}). | Empirical/Declared |
| `std.cmp::ge` | fn | `lib/std/cmp.myc:93` | `fn ge{N}(a: Binary{N}, b: Binary{N}) => Bool` | ge<N>: True iff a >= b — a projection of cmp (Exact; total). | Empirical/Declared |
| `std.cmp::max` | fn | `lib/std/cmp.myc:98` | `fn max{N}(a: Binary{N}, b: Binary{N}) => Binary{N}` | max<N>: the larger of a and b; returns b when equal (arbitrary but total — equal Binary{N} values are bit-identical, so the tie-break is observationally irrelevant). Exact. | Empirical/Declared |
| `std.cmp::min` | fn | `lib/std/cmp.myc:102` | `fn min{N}(a: Binary{N}, b: Binary{N}) => Binary{N}` | min<N>: the smaller of a and b; returns a when equal (irrelevant tie-break, as max). Exact. | Empirical/Declared |
| `std.cmp::ne` | fn | `lib/std/cmp.myc:111` | `fn ne{N}(a: Binary{N}, b: Binary{N}) => Bool` | ne<N>: True iff a != b — the negation of `eq` (Exact; total). | Empirical/Declared |
| `std.cmp::gt` | fn | `lib/std/cmp.myc:115` | `fn gt{N}(a: Binary{N}, b: Binary{N}) => Bool` | gt<N>: True iff a > b (unsigned) — a projection of cmp (Exact; total). | Empirical/Declared |
| `std.cmp::cmp_s` | fn | `lib/std/cmp.myc:121` | `fn cmp_s{N}(a: Binary{N}, b: Binary{N}) => Ordering` | cmp_s<N>: SIGNED three-way comparison — two's-complement ordering (RFC-0033 §4.1.2: ordering is signedness-distinct). Uses the `lt_s` prim (signed less-than) + `eq` (bit-identity, signedness- agnostic). Total, Exact for every concrete N. | Empirical/Declared |
| `std.cmp::le_s` | fn | `lib/std/cmp.myc:125` | `fn le_s{N}(a: Binary{N}, b: Binary{N}) => Bool` | le_s<N>: True iff a <= b under the SIGNED (two's-complement) ordering — projection of cmp_s. Exact. | Empirical/Declared |
| `std.cmp::ge_s` | fn | `lib/std/cmp.myc:129` | `fn ge_s{N}(a: Binary{N}, b: Binary{N}) => Bool` | ge_s<N>: True iff a >= b under the SIGNED ordering — projection of cmp_s. Exact. | Empirical/Declared |

### std.collections

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.collections` | nodule | `lib/std/collections.myc:8` | `nodule std.collections` | Self-hosted collection types — Vec (cons-list), Map (association list), Set (key list) | Empirical/Declared |
| `std.collections::Option` | type | `lib/std/collections.myc:27` | `type Option[Opt] = Some(Opt) \| None` | with honest O(n) bounds and never-silent fallibility. M-716 (E13-1, #461). Guarantee (per-op, audit-precise): constructors + total discriminators are Exact; eliminating/transforming ops carry a Declared type-level contract; `len`'s "fits Binary{8}" bound is Empirical (add_u refuses at 256); three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT) is Empirical (trials, std_collections.rs). Never-silent (G2): every fallible op returns Option — no panic, no sentinel. Map/Set lookup use the `eq` kernel prim (width-typed over Binary{N}): map_get and set_contains are now WIDTH-GENERIC over the key/element width `N` (M-718/M-753 — the recursive linear scan is itself width-generic, enabled by the width-var pass-through in checkty.rs `unify`), superseding the wave-n1 Binary{8}-monomorphic interim. Vec ops are fully generic except len/get which use Binary{8} as the index width. Option re-declaration: collections.myc is a self-contained nodule — test drivers append it verbatim and expect Option and Bool in scope without a separate import step (mirrors option.myc and result.myc self-containedness). The type parameter is named `Opt` (not `A`) to avoid substitution-key collisions when Option is nested inside Vec<A> (e.g. Option<Vec<A>>) — the checker's generic-constructor inference uses string-keyed substitution, and two type params with the same name `A` from different generic types collide (RFC-0007 §11.3 + M-716). The external name is unchanged: callers write `Option<Binary{8}>` as always. | Empirical/Declared |
| `std.collections::Option::None` | ctor | `lib/std/collections.myc:27` | `None` | — | Empirical/Declared |
| `std.collections::Option::Some` | ctor | `lib/std/collections.myc:27` | `Some(Opt)` | — | Empirical/Declared |
| `std.collections::Vec` | type | `lib/std/collections.myc:35` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | Rep: type Vec<A> = Nil \| Cons(A, Vec<A>) All ops are O(n) in the spine (Declared). Constructors Nil/Cons are Exact (total). `len` is O(n) and its "fits Binary{8}" contract is Empirical: add_u refuses at 256 on all paths. `get` is O(n) and OOB → None (never-silent, G2). | Empirical/Declared |
| `std.collections::Vec::Cons` | ctor | `lib/std/collections.myc:35` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.collections::Vec::Nil` | ctor | `lib/std/collections.myc:35` | `Nil` | — | Empirical/Declared |
| `std.collections::empty` | fn | `lib/std/collections.myc:38` | `fn empty[A]() => Vec[A]` | empty: the empty Vec. Total constructor (Exact). | Empirical/Declared |
| `std.collections::push_front` | fn | `lib/std/collections.myc:43` | `fn push_front[A](x: A, xs: Vec[A]) => Vec[A]` | push_front: prepend `x` to `xs`. O(1) cons (Exact constructor; the "O(1)" claim is Declared — one cons allocation, not an amortised bound — no asymptotic theorem is checked). | Empirical/Declared |
| `std.collections::is_empty` | fn | `lib/std/collections.myc:47` | `fn is_empty[A](xs: Vec[A]) => Bool` | is_empty: True iff the Vec is Nil. Total (Exact). | Empirical/Declared |
| `std.collections::head` | fn | `lib/std/collections.myc:52` | `fn head[A](xs: Vec[A]) => Option[A]` | head: the first element, or None on an empty Vec. Never-silent (G2): Nil → None, not a panic or sentinel. Declared (type-level contract; not a theorem). | Empirical/Declared |
| `std.collections::tail` | fn | `lib/std/collections.myc:56` | `fn tail[A](xs: Vec[A]) => Option[Vec[A]]` | tail: the rest after the first element, or None on empty. Never-silent (G2): Nil → None. Declared. | Empirical/Declared |
| `std.collections::len` | fn | `lib/std/collections.myc:63` | `fn len[A](xs: Vec[A]) => Binary{8}` | len: the number of elements. O(n) recursive spine-walk. Uses add_u at Binary{8}: a list longer than 255 elements causes add_u to refuse on ALL paths — never a silent wrap (G2). This is an explicit never-silent contract; the "fits Binary{8}" bound is Empirical (add_u's runtime refusal is the evidence, not a type-level proof). Declared (type-level contract for the O(n) walk). | Empirical/Declared |
| `std.collections::get` | fn | `lib/std/collections.myc:68` | `fn get[A](xs: Vec[A], i: Binary{8}) => Option[A]` | get: the element at index `i` (Binary{8}), or None if OOB. O(n) spine-walk. Never-silent (G2): OOB → None, never a fabricated value. Declared. | Empirical/Declared |
| `std.collections::snoc` | fn | `lib/std/collections.myc:84` | `fn snoc[A](xs: Vec[A], x: A) => Vec[A]` | snoc: append `x` at the end of `xs`. O(n) spine-walk to the tail (Declared). The spine is rebuilt on the way back — a pure value-semantics copy. Base case: push_front(x, xs) where xs = Nil (the matched branch), which is Cons(x, Nil) by evaluation. We write push_front(x, xs) rather than Cons(x, Nil) because bare nullary constructors (Nil) cannot appear in field position when the type parameter A is still abstract — the checker requires a concrete expected context (RFC-0007 §11.3). push_front(x, xs) synthesizes the correct Vec<A> type without that constraint. | Empirical/Declared |
| `std.collections::reverse` | fn | `lib/std/collections.myc:95` | `fn reverse[A](xs: Vec[A]) => Vec[A]` | reverse: reverse `xs` via recursive snoc (O(n²): each snoc is O(n), called n times). The O(n) accumulator-based reversal requires passing a bare `Nil` as a generic function argument, which cannot be type-inferred when the type parameter is still abstract (RFC-0007 §11.3 — the checker requires an expected context for nullary generic constructors; a call-site argument position does not propagate one when the parameter type has a free variable). This O(n²) form avoids that constraint and is honest about the cost. Declared (O(n²) structural recursion). FLAG: an O(n) reverse using a typed `empty()` helper or let-binding may land when RFC-0007 §11.3 ascription syntax or a type-argument inference extension is available. | Empirical/Declared |
| `std.collections::Map` | type | `lib/std/collections.myc:104` | `type Map[K, V] = MNil \| MCons(K, V, Map[K, V])` | Rep: MNil \| MCons(k, v, Map<K,V>) Lookup uses the `eq` kernel prim (width-typed). map_get is WIDTH-GENERIC over the key width `N` (M-718/M-753) and fully generic over the value type `V`. Missing key → None (never-silent, G2). All ops are O(n) (Declared). | Empirical/Declared |
| `std.collections::Map::MCons` | ctor | `lib/std/collections.myc:104` | `MCons(K, V, Map[K, V])` | — | Empirical/Declared |
| `std.collections::Map::MNil` | ctor | `lib/std/collections.myc:104` | `MNil` | — | Empirical/Declared |
| `std.collections::map_empty` | fn | `lib/std/collections.myc:107` | `fn map_empty[K, V]() => Map[K, V]` | map_empty: the empty Map. Total constructor (Exact). | Empirical/Declared |
| `std.collections::map_insert` | fn | `lib/std/collections.myc:112` | `fn map_insert[K, V](k: K, v: V, m: Map[K, V]) => Map[K, V]` | map_insert: insert key `k` with value `v` at the head of the association list. O(1) cons (Exact constructor). Does NOT deduplicate keys — the first occurrence wins on lookup (Declared). | Empirical/Declared |
| `std.collections::map_get` | fn | `lib/std/collections.myc:119` | `fn map_get[V]{N}(m: Map[Binary{N}, V], k: Binary{N}) => Option[V]` | map_get: O(n) linear scan for key `k` in `m`. Returns Some(v) on the first match, None on miss. Never-silent (G2): missing key → None, not a sentinel or panic. Width-GENERIC over the key width `N` and fully generic over the value type `V` (M-718/M-753): the key is `Binary{N}` (compared by the width-typed `eq` prim), the value `V` is opaque (only carried). Declared. | Empirical/Declared |
| `std.collections::Set` | type | `lib/std/collections.myc:130` | `type Set[A] = SNil \| SCons(A, Set[A])` | Rep: SNil \| SCons(x, Set<A>) set_contains is WIDTH-GENERIC over the element width `N` (M-718/M-753) — same width-typed `eq` constraint as map_get. All ops are O(n) (Declared). Constructors Exact. | Empirical/Declared |
| `std.collections::Set::SCons` | ctor | `lib/std/collections.myc:130` | `SCons(A, Set[A])` | — | Empirical/Declared |
| `std.collections::Set::SNil` | ctor | `lib/std/collections.myc:130` | `SNil` | — | Empirical/Declared |
| `std.collections::set_empty` | fn | `lib/std/collections.myc:133` | `fn set_empty[A]() => Set[A]` | set_empty: the empty Set. Total constructor (Exact). | Empirical/Declared |
| `std.collections::set_add` | fn | `lib/std/collections.myc:138` | `fn set_add[A](x: A, s: Set[A]) => Set[A]` | set_add: add element `x` to the front of the Set. O(1) cons (Exact constructor). Does NOT deduplicate — a key may appear more than once; set_contains returns True on first match (Declared). | Empirical/Declared |
| `std.collections::set_contains` | fn | `lib/std/collections.myc:144` | `fn set_contains{N}(s: Set[Binary{N}], x: Binary{N}) => Bool` | set_contains: O(n) linear scan — True iff `x` appears in `s`. Never-silent (G2): absent → False, not a sentinel. Width-GENERIC over the element width `N` (M-718/M-753): the element is `Binary{N}`, compared by the width-typed `eq` prim, monomorphized to the call-site width. Declared. | Empirical/Declared |

### std.core

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.core` | nodule | `lib/std/core.myc:8` | `nodule std.core` | Self-hosted RFC-0016 §4.5 `std.core` guarantee matrix — the Ring-0 prelude's 9-row | Empirical/Declared |
| `std.core::Guarantee` | type | `lib/std/core.myc:82` | `type Guarantee = GExact \| GProven \| GEmpirical \| GDeclared` | Naming (FLAG, surface note): `Exact`/`Proven`/`Empirical`/`Declared` are RESERVED (the `T @ Exact` guarantee-strength type-annotation surface, `token.rs::StrengthTok`). Constructors are prefixed `G` (`GExact`/`GProven`/`GEmpirical`/`GDeclared`) to avoid the keyword collision. | Empirical/Declared |
| `std.core::Guarantee::GDeclared` | ctor | `lib/std/core.myc:82` | `GDeclared` | — | Empirical/Declared |
| `std.core::Guarantee::GEmpirical` | ctor | `lib/std/core.myc:82` | `GEmpirical` | — | Empirical/Declared |
| `std.core::Guarantee::GExact` | ctor | `lib/std/core.myc:82` | `GExact` | — | Empirical/Declared |
| `std.core::Guarantee::GProven` | ctor | `lib/std/core.myc:82` | `GProven` | — | Empirical/Declared |
| `std.core::GuaranteeRow` | type | `lib/std/core.myc:90` | `type GuaranteeRow = Row(Bytes, Guarantee, Bytes, Bytes, Bool)` | — | Empirical/Declared |
| `std.core::GuaranteeRow::Row` | ctor | `lib/std/core.myc:90` | `Row(Bytes, Guarantee, Bytes, Bytes, Bool)` | — | Empirical/Declared |
| `std.core::row_op` | fn | `lib/std/core.myc:93` | `fn row_op(r: GuaranteeRow) => Bytes` | — | Empirical/Declared |
| `std.core::row_tag` | fn | `lib/std/core.myc:96` | `fn row_tag(r: GuaranteeRow) => Guarantee` | — | Empirical/Declared |
| `std.core::row_fallibility` | fn | `lib/std/core.myc:99` | `fn row_fallibility(r: GuaranteeRow) => Bytes` | — | Empirical/Declared |
| `std.core::row_effects` | fn | `lib/std/core.myc:102` | `fn row_effects(r: GuaranteeRow) => Bytes` | — | Empirical/Declared |
| `std.core::row_explainable` | fn | `lib/std/core.myc:105` | `fn row_explainable(r: GuaranteeRow) => Bool` | — | Empirical/Declared |
| `std.core::is_exact` | fn | `lib/std/core.myc:109` | `fn is_exact(g: Guarantee) => Bool` | — | Empirical/Declared |
| `std.core::bool_and` | fn | `lib/std/core.myc:114` | `fn bool_and(a: Bool, b: Bool) => Bool` | bool_and / bool_not: local total Bool combinators (no cross-leaf import; the std.diag/std.cmp self-containedness convention). Total (Exact). | Empirical/Declared |
| `std.core::bool_not` | fn | `lib/std/core.myc:117` | `fn bool_not(a: Bool) => Bool` | — | Empirical/Declared |
| `std.core::byte_is` | fn | `lib/std/core.myc:123` | `fn byte_is(b: Bytes, i: Binary{8}, v: Binary{8}) => Bool` | byte_is: True iff byte `i` of `b` equals `v`. Delegates to the Exact `bytes_get`/`eq` prims. PRECONDITION (caller-guarded, never violated below): `i < bytes_len(b)` — `bytes_get` is explicit-error out of range (C1 never-silent), so every call site length-guards first. | Empirical/Declared |
| `std.core::effects_is_none` | fn | `lib/std/core.myc:130` | `fn effects_is_none(b: Bytes) => Bool` | effects_is_none: True iff `b` is EXACTLY the 4 bytes "none" — the full-content port of the Rust test's `row.effects == "none"` assertion (see the Bytes-comparison note above). The length guard makes the byte reads total: the byte comparisons run only in the len==4 arm. 'n' = 0x6E, 'o' = 0x6F, 'e' = 0x65. Total (Exact by composition of Exact prims). | Empirical/Declared |
| `std.core::row_value_repr_meta` | fn | `lib/std/core.myc:145` | `fn row_value_repr_meta() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::row_corevalue_datum` | fn | `lib/std/core.myc:148` | `fn row_corevalue_datum() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::row_guarantee_strength` | fn | `lib/std/core.myc:151` | `fn row_guarantee_strength() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::row_bound_boundbasis` | fn | `lib/std/core.myc:154` | `fn row_bound_boundbasis() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::row_repr_of` | fn | `lib/std/core.myc:158` | `fn row_repr_of() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::row_meta_of` | fn | `lib/std/core.myc:161` | `fn row_meta_of() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::row_guarantee_of` | fn | `lib/std/core.myc:164` | `fn row_guarantee_of() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::row_bound_of` | fn | `lib/std/core.myc:167` | `fn row_bound_of() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::row_provenance_of` | fn | `lib/std/core.myc:170` | `fn row_provenance_of() => GuaranteeRow` | — | Empirical/Declared |
| `std.core::Vec` | type | `lib/std/core.myc:174` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.core::Vec::Cons` | ctor | `lib/std/core.myc:174` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.core::Vec::Nil` | ctor | `lib/std/core.myc:174` | `Nil` | — | Empirical/Declared |
| `std.core::matrix` | fn | `lib/std/core.myc:177` | `fn matrix() => Vec[GuaranteeRow]` | matrix: the full 9-row table, in the same order as mycelium_std_core::GUARANTEE_MATRIX. | Empirical/Declared |
| `std.core::matrix_len` | fn | `lib/std/core.myc:195` | `fn matrix_len(xs: Vec[GuaranteeRow]) => Binary{8}` | — | Empirical/Declared |
| `std.core::all_exact` | fn | `lib/std/core.myc:201` | `fn all_exact(xs: Vec[GuaranteeRow]) => Bool` | all_exact: True iff every row's tag is Exact — one half of lib.rs::tests::matrix_is_all_exact_and_effect_free ("a Proven/Empirical tag here would itself violate VR-5"). | Empirical/Declared |
| `std.core::all_effects_none` | fn | `lib/std/core.myc:207` | `fn all_effects_none(xs: Vec[GuaranteeRow]) => Bool` | all_effects_none: True iff every row's effects field is EXACTLY "none" — the other half of lib.rs::tests::matrix_is_all_exact_and_effect_free, ported at FULL content fidelity (see effects_is_none above). | Empirical/Declared |
| `std.core::explainable_count` | fn | `lib/std/core.myc:215` | `fn explainable_count(xs: Vec[GuaranteeRow]) => Binary{8}` | explainable_count: how many rows surface an inspectable EXPLAIN artifact, as Binary{8} — the cardinality half of lib.rs::tests::only_query_rows_are_explainable. | Empirical/Declared |
| `std.core::only_query_rows_explainable` | fn | `lib/std/core.myc:228` | `fn only_query_rows_explainable() => Bool` | only_query_rows_explainable: the EXPLAIN window is exactly the value-tag/bound/provenance queries — lib.rs::tests::only_query_rows_are_explainable, ported as a per-row identity check (the Rust test's op-NAME list equality needs bytes_eq — the identity of each row constructor carries the same information here, so nothing is lost). | Empirical/Declared |

### std.diag

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.diag` | nodule | `lib/std/diag.myc:8` | `nodule std.diag` | Self-hosted RFC-0016 §4.5 `std.diag` guarantee matrix — the 14-row honest-tag/ | Empirical/Declared |
| `std.diag::Guarantee` | type | `lib/std/diag.myc:59` | `type Guarantee = GExact \| GProven \| GEmpirical \| GDeclared` | Naming (FLAG, surface note): `Exact`/`Proven`/`Empirical`/`Declared` are RESERVED (the `T @ Exact` guarantee-strength type-annotation surface, `token.rs::StrengthTok` — a distinct, type-level mechanism from this VALUE-level lattice-tag data). Constructors here are prefixed `G` (`GExact`/`GProven`/`GEmpirical`/`GDeclared`) to avoid the keyword collision. | Empirical/Declared |
| `std.diag::Guarantee::GDeclared` | ctor | `lib/std/diag.myc:59` | `GDeclared` | — | Empirical/Declared |
| `std.diag::Guarantee::GEmpirical` | ctor | `lib/std/diag.myc:59` | `GEmpirical` | — | Empirical/Declared |
| `std.diag::Guarantee::GExact` | ctor | `lib/std/diag.myc:59` | `GExact` | — | Empirical/Declared |
| `std.diag::Guarantee::GProven` | ctor | `lib/std/diag.myc:59` | `GProven` | — | Empirical/Declared |
| `std.diag::Fallibility` | type | `lib/std/diag.myc:64` | `type Fallibility = Total \| Explicit` | — | Empirical/Declared |
| `std.diag::Fallibility::Explicit` | ctor | `lib/std/diag.myc:64` | `Explicit` | — | Empirical/Declared |
| `std.diag::Fallibility::Total` | ctor | `lib/std/diag.myc:64` | `Total` | — | Empirical/Declared |
| `std.diag::Explainable` | type | `lib/std/diag.myc:68` | `type Explainable = IsExplainRecord \| ContentAddressedHandle \| NotApplicable \| ClosedVocabulary` | — | Empirical/Declared |
| `std.diag::Explainable::ClosedVocabulary` | ctor | `lib/std/diag.myc:68` | `ClosedVocabulary` | — | Empirical/Declared |
| `std.diag::Explainable::ContentAddressedHandle` | ctor | `lib/std/diag.myc:68` | `ContentAddressedHandle` | — | Empirical/Declared |
| `std.diag::Explainable::IsExplainRecord` | ctor | `lib/std/diag.myc:68` | `IsExplainRecord` | — | Empirical/Declared |
| `std.diag::Explainable::NotApplicable` | ctor | `lib/std/diag.myc:68` | `NotApplicable` | — | Empirical/Declared |
| `std.diag::MatrixRow` | type | `lib/std/diag.myc:75` | `type MatrixRow = Row(Bytes, Guarantee, Fallibility, Bytes, Bytes, Explainable, Bytes)` | — | Empirical/Declared |
| `std.diag::MatrixRow::Row` | ctor | `lib/std/diag.myc:75` | `Row(Bytes, Guarantee, Fallibility, Bytes, Bytes, Explainable, Bytes)` | — | Empirical/Declared |
| `std.diag::row_op` | fn | `lib/std/diag.myc:78` | `fn row_op(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.diag::row_guarantee` | fn | `lib/std/diag.myc:81` | `fn row_guarantee(r: MatrixRow) => Guarantee` | — | Empirical/Declared |
| `std.diag::row_fallibility` | fn | `lib/std/diag.myc:84` | `fn row_fallibility(r: MatrixRow) => Fallibility` | — | Empirical/Declared |
| `std.diag::row_error_set` | fn | `lib/std/diag.myc:87` | `fn row_error_set(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.diag::row_effects` | fn | `lib/std/diag.myc:90` | `fn row_effects(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.diag::row_explainable` | fn | `lib/std/diag.myc:93` | `fn row_explainable(r: MatrixRow) => Explainable` | — | Empirical/Declared |
| `std.diag::row_never_silent` | fn | `lib/std/diag.myc:96` | `fn row_never_silent(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.diag::is_exact` | fn | `lib/std/diag.myc:100` | `fn is_exact(g: Guarantee) => Bool` | — | Empirical/Declared |
| `std.diag::is_total` | fn | `lib/std/diag.myc:103` | `fn is_total(f: Fallibility) => Bool` | — | Empirical/Declared |
| `std.diag::is_explicit` | fn | `lib/std/diag.myc:106` | `fn is_explicit(f: Fallibility) => Bool` | — | Empirical/Declared |
| `std.diag::is_explain_record` | fn | `lib/std/diag.myc:109` | `fn is_explain_record(x: Explainable) => Bool` | — | Empirical/Declared |
| `std.diag::is_content_addressed_handle` | fn | `lib/std/diag.myc:117` | `fn is_content_addressed_handle(x: Explainable) => Bool` | — | Empirical/Declared |
| `std.diag::bool_and` | fn | `lib/std/diag.myc:127` | `fn bool_and(a: Bool, b: Bool) => Bool` | bool_and: local total conjunction (no cross-leaf import; mirrors std.cmp's bool_eq/bool_cmp self-containedness convention). Total over Bool x Bool (Exact). | Empirical/Declared |
| `std.diag::nonempty` | fn | `lib/std/diag.myc:132` | `fn nonempty(b: Bytes) => Bool` | nonempty: True iff `b` has at least one byte. Delegates to the Exact `bytes_len` prim; the zero-comparison is the Exact `eq` prim at Binary{32} (the width `bytes_len` returns). Total (Exact). | Empirical/Declared |
| `std.diag::row_present` | fn | `lib/std/diag.myc:139` | `fn row_present() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_content_id` | fn | `lib/std/diag.myc:150` | `fn row_content_id() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_to_human` | fn | `lib/std/diag.myc:162` | `fn row_to_human() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_to_json` | fn | `lib/std/diag.myc:173` | `fn row_to_json() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_from_json` | fn | `lib/std/diag.myc:184` | `fn row_from_json() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_resolve_class` | fn | `lib/std/diag.myc:196` | `fn row_resolve_class() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_register_class` | fn | `lib/std/diag.myc:207` | `fn row_register_class() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_on_policy` | fn | `lib/std/diag.myc:219` | `fn row_on_policy() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_policy_ref` | fn | `lib/std/diag.myc:230` | `fn row_policy_ref() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_from_file` | fn | `lib/std/diag.myc:241` | `fn row_from_file() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_resolve_route` | fn | `lib/std/diag.myc:253` | `fn row_resolve_route() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_sink` | fn | `lib/std/diag.myc:264` | `fn row_sink() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_guarantee_of` | fn | `lib/std/diag.myc:276` | `fn row_guarantee_of() => MatrixRow` | — | Empirical/Declared |
| `std.diag::row_audit_of` | fn | `lib/std/diag.myc:287` | `fn row_audit_of() => MatrixRow` | — | Empirical/Declared |
| `std.diag::Vec` | type | `lib/std/diag.myc:299` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.diag::Vec::Cons` | ctor | `lib/std/diag.myc:299` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.diag::Vec::Nil` | ctor | `lib/std/diag.myc:299` | `Nil` | — | Empirical/Declared |
| `std.diag::matrix` | fn | `lib/std/diag.myc:302` | `fn matrix() => Vec[MatrixRow]` | matrix: the full 14-row table, in the same order as guarantee_matrix.rs::MATRIX. | Empirical/Declared |
| `std.diag::matrix_len` | fn | `lib/std/diag.myc:324` | `fn matrix_len(xs: Vec[MatrixRow]) => Binary{8}` | — | Empirical/Declared |
| `std.diag::all_exact` | fn | `lib/std/diag.myc:328` | `fn all_exact(xs: Vec[MatrixRow]) => Bool` | all_exact: True iff every row's guarantee is Exact (guarantee_matrix.rs::all_diag_ops_are_exact). | Empirical/Declared |
| `std.diag::all_never_silent_nonempty` | fn | `lib/std/diag.myc:333` | `fn all_never_silent_nonempty(xs: Vec[MatrixRow]) => Bool` | all_never_silent_nonempty: True iff every row states a non-empty never_silent_property (guarantee_matrix.rs::every_row_states_never_silent_property). | Empirical/Declared |
| `std.diag::all_effects_nonempty` | fn | `lib/std/diag.myc:340` | `fn all_effects_nonempty(xs: Vec[MatrixRow]) => Bool` | all_effects_nonempty: True iff every row states its effects (guarantee_matrix.rs::every_row_states_effects). | Empirical/Declared |
| `std.diag::explicit_ops_have_error_set` | fn | `lib/std/diag.myc:348` | `fn explicit_ops_have_error_set(xs: Vec[MatrixRow]) => Bool` | explicit_ops_have_error_set: True iff every Explicit-fallibility row states a non-empty error_set (guarantee_matrix.rs::explicit_ops_have_nonempty_error_set). A Total row is vacuously True. | Empirical/Declared |
| `std.diag::present_is_i1_crux` | fn | `lib/std/diag.myc:361` | `fn present_is_i1_crux() => Bool` | present_is_i1_crux: the structural (non-substring) half of guarantee_matrix.rs::present_is_the_i1_crux — present is Total, Exact, and IsExplainRecord. (The "error_set contains UNCHANGED" substring assertion is FLAGged above as not portable — no bytes_eq/contains prim.) | Empirical/Declared |
| `std.diag::content_id_and_policy_ref_are_handles` | fn | `lib/std/diag.myc:368` | `fn content_id_and_policy_ref_are_handles() => Bool` | content_id_and_policy_ref_are_handles: guarantee_matrix.rs::content_id_and_policy_ref_are_content_addressed_handles — both rows are ContentAddressedHandle (fully portable: pure ADT equality, no substring needed). | Empirical/Declared |

### std.error

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.error` | nodule | `lib/std/error.myc:8` | `nodule std.error` | Self-hosted errors-as-values ergonomics layer over Result<A,E>/Option<A> — propagate, | Empirical/Declared |
| `std.error::Result` | type | `lib/std/error.myc:82` | `type Result[A, E] = Ok(A) \| Err(E)` | Never-silent floor (C1/I1) carried over intact: every ported op either transforms the sum (the error/absence survives in the result), re-propagates it, or explicitly recovers it with an honest tag (never upgraded — VR-5). No ported op silently drops an `Err`/`None`. Result/Option — local mirrors of std.result/std.option (see the self-containment note above). | Empirical/Declared |
| `std.error::Result::Err` | ctor | `lib/std/error.myc:82` | `Err(E)` | — | Empirical/Declared |
| `std.error::Result::Ok` | ctor | `lib/std/error.myc:82` | `Ok(A)` | — | Empirical/Declared |
| `std.error::Option` | type | `lib/std/error.myc:84` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `std.error::Option::None` | ctor | `lib/std/error.myc:84` | `None` | — | Empirical/Declared |
| `std.error::Option::Some` | ctor | `lib/std/error.myc:84` | `Some(A)` | — | Empirical/Declared |
| `std.error::Unit` | type | `lib/std/error.myc:88` | `type Unit = U` | Unit — a local nullary-marker substitute for the missing `() => T` closure-domain form (see the substitution note above). A single-constructor, no-field type — not a new \*kind\* of type (KC-3). | Empirical/Declared |
| `std.error::Unit::U` | ctor | `lib/std/error.myc:88` | `U` | — | Empirical/Declared |
| `std.error::map` | fn | `lib/std/error.myc:96` | `fn map[A, B, E](r: Result[A, E], f: A => B) => Result[B, E]` | map/map_err/and_then/or_else duplicate std.result's landed combinators of the same name (see the self-containment note above) — same semantics, same guarantee (Declared: type-level contract; differential agreement Empirical, std_error.rs), carried at the SAME strength (VR-5). map: apply `f` to the success value; an Err passes through untouched (error preserved in sum). | Empirical/Declared |
| `std.error::map_err` | fn | `lib/std/error.myc:100` | `fn map_err[A, E, F](r: Result[A, E], f: E => F) => Result[A, F]` | map_err: transform the error; an Ok passes through untouched (the Err-side mirror of map). | Empirical/Declared |
| `std.error::and_then` | fn | `lib/std/error.myc:105` | `fn and_then[A, B, E](r: Result[A, E], f: A => Result[B, E]) => Result[B, E]` | and_then: monadic bind — apply `f` to the success value; an Err short-circuits and propagates (never dropped). | Empirical/Declared |
| `std.error::or_else` | fn | `lib/std/error.myc:110` | `fn or_else[A, E](r: Result[A, E], f: E => Result[A, E]) => Result[A, E]` | or_else: explicit recovery hook — apply `f` to the error; an Ok passes through. `f` must yield a Result (recover or re-propagate — never a drop). | Empirical/Declared |
| `std.error::filter` | fn | `lib/std/error.myc:115` | `fn filter[A](o: Option[A], predicate: A => Bool) => Option[A]` | filter: Some(x) where predicate(x) is False becomes None — a typed transition (named absence), not a silent loss (I1/C1). | Empirical/Declared |
| `std.error::inspect` | fn | `lib/std/error.myc:123` | `fn inspect[A, E, B](r: Result[A, E], f: A => B) => Result[A, E]` | inspect: peek the Ok side; the value and sum shape are unchanged. `.myc` closures carry no declared-effect annotations in HOF stage 1 (RFC-0024 §3) — unlike the Rust crate's `FnOnce(&T)`, `f`'s return value is computed and then discarded (never observed further), so this combinator is honestly a structural peek, not an effect-tracked one (no RFC-0016 C6 obligation to discharge since `.myc` has no closure-effect surface yet — captured, not faked). | Empirical/Declared |
| `std.error::inspect_err` | fn | `lib/std/error.myc:132` | `fn inspect_err[A, E, B](r: Result[A, E], f: E => B) => Result[A, E]` | inspect_err: peek the Err side; the value and propagation are unchanged (mirror of inspect). | Empirical/Declared |
| `std.error::ok_or` | fn | `lib/std/error.myc:142` | `fn ok_or[A, E](o: Option[A], err: E) => Result[A, E]` | — | Empirical/Declared |
| `std.error::ok_or_else` | fn | `lib/std/error.myc:147` | `fn ok_or_else[A, E](o: Option[A], f: Unit => E) => Result[A, E]` | ok_or_else: None becomes Err(f(U)) — a lazily-computed error (see the Unit substitution note above; f is only invoked on the None arm, so laziness is preserved). | Empirical/Declared |
| `std.error::ok` | fn | `lib/std/error.myc:155` | `fn ok[A, E](r: Result[A, E]) => Option[A]` | ok: Result -> Option. FLAGGED LOSSY CONVERSION (spec §7-Q2/C3): the one op in this surface that discards `E` (Err -> None). C1-honest only because it is an explicitly-named, EXPLAIN-able lossy conversion (this doc comment), never an unflagged drop. Whether it should be gated behind an unmissable name (e.g. ok_discarding_err) awaits RFC-0016 §8-Q3 ratification (spec FLAG Q2, carried over unresolved — not this port's call to make). | Empirical/Declared |
| `std.error::flatten` | fn | `lib/std/error.myc:160` | `fn flatten[A, E](r: Result[Result[A, E], E]) => Result[A, E]` | flatten: collapse a nested Result; the inner Err propagates to the outer — no wrapping is discarded silently. | Empirical/Declared |
| `std.error::unwrap_or` | fn | `lib/std/error.myc:172` | `fn unwrap_or[A, E](r: Result[A, E], fallback: A) => A` | unwrap_or/unwrap_or_else duplicate std.result's landed combinators of the same name (Result side); unwrap_or_option/unwrap_or_else_option duplicate std.option's `unwrap_or`/generalize it with a lazy default (Option side) — see the self-containment note above. Guarantee: Declared (RFC-0014 I2/VR-5 — the substituted default is asserted, not proven; downgrade is the rule). Never-silent (I1/C1): the caller supplies the default/closure explicitly — no panic, no sentinel. unwrap_or: the success value, else the caller-supplied fallback. | Empirical/Declared |
| `std.error::unwrap_or_else` | fn | `lib/std/error.myc:176` | `fn unwrap_or_else[A, E](r: Result[A, E], f: E => A) => A` | unwrap_or_else: the success value, else a computed fallback from the error. | Empirical/Declared |
| `std.error::unwrap_or_option` | fn | `lib/std/error.myc:180` | `fn unwrap_or_option[A](o: Option[A], fallback: A) => A` | unwrap_or_option: the held value, else the caller-supplied fallback (Option variant). | Empirical/Declared |
| `std.error::unwrap_or_else_option` | fn | `lib/std/error.myc:185` | `fn unwrap_or_else_option[A](o: Option[A], f: Unit => A) => A` | unwrap_or_else_option: the held value, else a computed fallback (see the Unit substitution note above; f is only invoked on the None arm). | Empirical/Declared |

### std.fmt

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.fmt` | nodule | `lib/std/fmt.myc:8` | `nodule std.fmt` | Self-hosted first-order formatting: hex-digit conversion and two-digit hex encoding. | Empirical/Declared |
| `std.fmt::HexPair` | type | `lib/std/fmt.myc:27` | `type HexPair = HP(Binary{8}, Binary{8})` | of a Binary{8} byte. M-717 (E13-1, #462). Self-contained: no import of std.collections or std.text (not available in this worktree; must not be imported per worktree discipline). Guarantee: `hex_digit` is total over its finite nibble domain (Binary{8} inputs 0..15) — the case split is match-defined via `lt` + `add_u` making each in-range arm Exact; nibble >= 16 returns '?' (0x3F), never-silent (G2: the fallback is explicit, not a silent wrap). `nibble_lo(x)` is Exact (total bit-mask over Binary{8}). `nibble_hi(x)` is Exact over the 16 possible masked values via a 4-level lt binary-search match tree — implementable without a right-shift prim. `to_hex` is Declared (structural composition of nibble_hi/nibble_lo + hex_digit). Differential agreement (L1-eval ≡ L0-interp ≡ AOT) is Empirical (trials, std_fmt.rs). Honesty boundary: no reflective `display(Value)` — .myc has no Value introspection; fmt is first-order over concrete types only. Grounding: hand-computed values, three-way verified (L1≡L0≡AOT). crates/mycelium-std-fmt exists but exposes a different Ring-2 surface (no hex_digit/to_hex) — it is not the structural oracle. ── HexPair — the two ASCII hex-digit bytes of a byte encoding ──────────────────────────────────── HexPair: carries the high-nibble digit and the low-nibble digit as ASCII bytes (Binary{8}). Used as the return type of `to_hex` so the result is a first-class value pair. (A Bytes return would require bytes_concat which is not surface-callable — see FLAG-text-3 in text.myc.) | Empirical/Declared |
| `std.fmt::HexPair::HP` | ctor | `lib/std/fmt.myc:27` | `HP(Binary{8}, Binary{8})` | — | Empirical/Declared |
| `std.fmt::hex_digit` | fn | `lib/std/fmt.myc:39` | `fn hex_digit(nibble: Binary{8}) => Binary{8}` | Guarantee: Exact over the finite in-range domain (0..15) — the `lt`/`add_u` computation is a closed-form arithmetic on the finite range; Declared for the fallback arm (structurally necessary; unreachable for well-formed nibbles, never-silent for malformed ones). The `lt` prim returns Binary{1}; we use a match { 0b1 => …, _ => … } bridge. | Empirical/Declared |
| `std.fmt::nibble_lo` | fn | `lib/std/fmt.myc:48` | `fn nibble_lo(x: Binary{8}) => Binary{8}` | — | Empirical/Declared |
| `std.fmt::nibble_hi` | fn | `lib/std/fmt.myc:60` | `fn nibble_hi(x: Binary{8}) => Binary{8}` | The masked value `m = and(x, 0b1111_0000)` is one of exactly 16 values; every Binary{8} produces exactly one leaf via the 4-branch binary search (Exact). Each leaf returns the nibble value (masked 0x00 → nibble 0, 0x10 → nibble 1, …, 0xf0 → nibble 15) as a Binary{8} constant. | Empirical/Declared |
| `std.fmt::to_hex` | fn | `lib/std/fmt.myc:92` | `fn to_hex(x: Binary{8}) => HexPair` | — | Empirical/Declared |

### std.iter

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.iter` | nodule | `lib/std/iter.myc:8` | `nodule std.iter` | Self-hosted first-order iterator surface over the cons-list shape (List<A> = Nil \| | Empirical/Declared |
| `std.iter::Option` | type | `lib/std/iter.myc:23` | `type Option[Opt] = Some(Opt) \| None` | Cons(A, List<A>)). M-715 (E13-1, RFC-0031 §5 D4 Tier-0). Provides: is_empty_l, length, and the recursive HOF combinators map, filter, foldl, any, all, find — ALL now execute three-way (L1-eval ≡ L0-interp ≡ AOT). The recursive-HOF re-pass gap is CLOSED (M-715, rsm S3): a HOF parameter re-passed at a recursive call site (e.g. `map(rest, f)`) is threaded through as the SAME static specialization by mono::resolve_fn_args (RFC-0024 §4 defunctionalization, extended). Guarantee (per-op, audit-precise): List constructors and is_empty_l are Exact (total); length is Declared (O(n) walk) with Empirical add_u overflow; the HOF combinators carry Declared type-level contracts with Empirical three-way agreement (std_iter.rs). Never-silent (G2): find-miss → None; length overflow → refusal on all paths. Honesty boundary: HOF arguments use RFC-0024 static defunctionalization — a single NAMED top-level function arg; closures, multi-arg arrows, and partial application remain deferred (M-704). Option re-declaration: self-contained — Option/Bool in scope without import. The Option type parameter is named Opt (not A) to avoid substitution-key collisions when Option nests inside List<A> (RFC-0007 §11.3 + M-716). | Empirical/Declared |
| `std.iter::Option::None` | ctor | `lib/std/iter.myc:23` | `None` | — | Empirical/Declared |
| `std.iter::Option::Some` | ctor | `lib/std/iter.myc:23` | `Some(Opt)` | — | Empirical/Declared |
| `std.iter::List` | type | `lib/std/iter.myc:31` | `type List[A] = Nil \| Cons(A, List[A])` | Rep: type List<A> = Nil \| Cons(A, List<A>) Mirrors the Vec cons-list shape in std.collections. All ops are O(n) in the spine (Declared). Constructors are Exact (total). The name `List` is chosen to distinguish from `Vec` (which lives in std.collections) while sharing the same underlying cons-cell structure. | Empirical/Declared |
| `std.iter::List::Cons` | ctor | `lib/std/iter.myc:31` | `Cons(A, List[A])` | — | Empirical/Declared |
| `std.iter::List::Nil` | ctor | `lib/std/iter.myc:31` | `Nil` | — | Empirical/Declared |
| `std.iter::is_empty_l` | fn | `lib/std/iter.myc:34` | `fn is_empty_l[A](xs: List[A]) => Bool` | is_empty_l: True iff the List is Nil. Total (Exact). | Empirical/Declared |
| `std.iter::map` | fn | `lib/std/iter.myc:44` | `fn map[A, B](xs: List[A], f: A => B) => List[B]` | map: apply `f` to every element; structure is preserved. O(n) spine-walk (Declared). The function `f` is a single-arg named function (RFC-0024 defunctionalization). Never-silent (G2): Nil passes through. Declared; three-way agreement Empirical (std_iter.rs). `f` is a single named top-level fn (RFC-0024 §4); the recursive call `map(rest, f)` re-passes `f`, now threaded through as the SAME static specialization (M-715 / mono::resolve_fn_args). Closures / multi-arg arrows: M-704. | Empirical/Declared |
| `std.iter::filter` | fn | `lib/std/iter.myc:53` | `fn filter[A](xs: List[A], pred: A => Bool) => List[A]` | filter: keep only elements satisfying `pred`; elements that fail are dropped. O(n) spine-walk (Declared). Never-silent (G2): dropped elements are explicit absences, not silent coercions. Declared; three-way agreement Empirical (M-715: the recursive re-pass of `pred` is threaded as the same static specialization). | Empirical/Declared |
| `std.iter::foldl` | fn | `lib/std/iter.myc:75` | `fn foldl[A, B](xs: List[A], f: A => B, acc: B) => B` | foldl: left-associative fold — reduce the list to a single B via `f` at each element. O(n) spine-walk (Declared). Nil returns the initial `acc` (never-silent, G2). `f: A -> B` is the single-arg HOF (RFC-0024 §3); the prior acc is replaced by `f(h)` at each step — not a true binary (A -> B -> B) fold (which requires multi-arg HOF, deferred RFC-0024 §5). Declared. Note: because each step discards the prior `acc` (it is `f(h)`, ignoring the accumulator), the effective result for a non-empty list is `f(last element)`; the initial `acc` is only ever returned on `Nil`. (A true binary fold — needing multi-arg HOF — is the deferred form below.) Three-way agreement Empirical (M-715: the recursive re-pass of `f` is threaded as the same static specialization). FLAG (still open): a TRUE two-arg fold (f: A -> B -> B, combining the accumulator) requires multi-arg HOF — deferred (M-704); this `f: A -> B` form discards the prior acc, as noted. | Empirical/Declared |
| `std.iter::length` | fn | `lib/std/iter.myc:85` | `fn length[A](xs: List[A]) => Binary{8}` | length: the number of elements. O(n) recursive spine-walk. Uses add_u at Binary{8}: a list longer than 255 elements causes add_u to refuse on ALL paths — never a silent wrap (G2). This is an explicit never-silent contract; the "fits Binary{8}" bound is Empirical (add_u's runtime refusal is the evidence, not a type-level proof). Declared (type-level contract for the O(n) walk). | Empirical/Declared |
| `std.iter::any` | fn | `lib/std/iter.myc:94` | `fn any[A](xs: List[A], pred: A => Bool) => Bool` | any: True iff any element satisfies `pred`. Short-circuits at the first True (O(n) worst-case, Declared). Never-silent (G2): empty → False (no fabricated True). Declared. Three-way agreement Empirical (M-715: the recursive re-pass of `pred` is threaded as the same static specialization). `pred` is a single named top-level fn (RFC-0024 §4; closures deferred, M-704). | Empirical/Declared |
| `std.iter::all` | fn | `lib/std/iter.myc:105` | `fn all[A](xs: List[A], pred: A => Bool) => Bool` | all: True iff every element satisfies `pred`. Short-circuits at the first False (O(n) worst-case, Declared). Never-silent (G2): empty → True (vacuous universal quantification — standard convention, not a fabricated result; documented to avoid confusion). Declared. Three-way agreement Empirical (M-715: the recursive re-pass of `pred` is threaded as the same static specialization). `pred` is a single named top-level fn (RFC-0024 §4; closures deferred, M-704). | Empirical/Declared |
| `std.iter::find` | fn | `lib/std/iter.myc:117` | `fn find[A](xs: List[A], pred: A => Bool) => Option[A]` | find: the first element satisfying `pred`, or None if no element qualifies. O(n) spine-walk (Declared). Never-silent (G2): a miss yields None — never a fabricated element, never a sentinel. Declared; three-way agreement Empirical (M-715: the recursive re-pass of `pred` is threaded as the same static specialization). | Empirical/Declared |

### std.math

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.math` | nodule | `lib/std/math.myc:7` | `nodule std.math` | Self-hosted width-generic arithmetic/logic surface — binary add/sub + bitwise and/or/xor/not | Empirical/Declared |
| `std.math::badd` | fn | `lib/std/math.myc:53` | `fn badd{N}(a: Binary{N}, b: Binary{N}) => Binary{N}` | Honesty boundary (FLAGs — VR-5; do NOT fake): FLAG-math-1 (UPDATED, CU-1/CU-6): binary MULTIPLY is now provided (`bmul` below) — the `bit.mul` never-silent unsigned-multiply prim landed (RFC-0033 §4.1.2). Binary/ternary DIVISION is still not surfaced here (the `bin.div`/`bin.rem` prims exist but the width-generic self-hosted division surface is a future increment); it is never faked. Ternary multiply rides `tmul` (kernel `trit.mul`). FLAG-math-3 (CU-6): bit ROTATE (`rotate_left`/`rotate_right`) and `reverse_bits`/`swap_bytes` are NOT provided width-generically. Rotate needs the width `N` as a runtime value to form the complementary shift `N - n`, and the naive `or(shl_u(x,n), shr_u(x, N-n))` mis-handles `n = 0` (a full-width `shr` refuses, shift-amount >= N) — so it is NOT a clean derivation from the surfaced prims. Faithful rotate is gated on either a dedicated `bit.rotl`/`bit.rotr` prim or a width-reflection surface; captured here, never faked (VR-5). popcount/clz/ctz ARE provided (`bpopcount`/`bclz`/`bctz`), the kernel `bit.popcount`/`bit.clz`/`bit.ctz` prims. FLAG-math-2: NO epsilon/delta numerics surface (honest rounding / precision disclosure over the Dense representation) is provided here. That surface (RFC-0031 §5 D4 Tier-2; the ε-propagation machinery, ApproxRule, in prims.rs) is a Dense-numerics workstream (E2-1/E2-4), not yet self-hostable from the surfaced prim set. Captured, not claimed. ── Binary arithmetic over Binary{N} (never-silent overflow/underflow) ───────────────────────────── badd<N>: unsigned width-preserving addition. Exact on the in-range result; an overflow (carry-out) is a never-silent `Overflow` refusal — never a modular wrap (G2). | Empirical/Declared |
| `std.math::bsub` | fn | `lib/std/math.myc:58` | `fn bsub{N}(a: Binary{N}, b: Binary{N}) => Binary{N}` | bsub<N>: unsigned width-preserving subtraction. Exact on the in-range result; an underflow (borrow-out below 0) is a never-silent `Overflow` refusal — never a wrap to a large value (G2). | Empirical/Declared |
| `std.math::band` | fn | `lib/std/math.myc:63` | `fn band{N}(a: Binary{N}, b: Binary{N}) => Binary{N}` | — | Empirical/Declared |
| `std.math::bor` | fn | `lib/std/math.myc:67` | `fn bor{N}(a: Binary{N}, b: Binary{N}) => Binary{N}` | bor<N>: bitwise OR. Total, Exact. | Empirical/Declared |
| `std.math::bxor` | fn | `lib/std/math.myc:71` | `fn bxor{N}(a: Binary{N}, b: Binary{N}) => Binary{N}` | bxor<N>: bitwise XOR. Total, Exact. | Empirical/Declared |
| `std.math::bnot` | fn | `lib/std/math.myc:75` | `fn bnot{N}(a: Binary{N}) => Binary{N}` | bnot<N>: bitwise NOT (one's complement). Total, Exact over Binary{N}. | Empirical/Declared |
| `std.math::bmul` | fn | `lib/std/math.myc:81` | `fn bmul{N}(a: Binary{N}, b: Binary{N}) => Binary{N}` | bmul<N>: unsigned width-preserving multiply (CU-1). Exact on the in-range result; a product that overflows `U_N` is a never-silent `Overflow` refusal — never a modular wrap (G2). Closes the FLAG-math-1 "no binary multiply" gap (kernel `bit.mul`). | Empirical/Declared |
| `std.math::bpopcount` | fn | `lib/std/math.myc:86` | `fn bpopcount{N}(a: Binary{N}) => Binary{N}` | — | Empirical/Declared |
| `std.math::bclz` | fn | `lib/std/math.myc:90` | `fn bclz{N}(a: Binary{N}) => Binary{N}` | bclz<N>: count leading zeros; N for the all-zero value. Total, Exact. | Empirical/Declared |
| `std.math::bctz` | fn | `lib/std/math.myc:94` | `fn bctz{N}(a: Binary{N}) => Binary{N}` | bctz<N>: count trailing zeros; N for the all-zero value. Total, Exact. | Empirical/Declared |
| `std.math::tadd` | fn | `lib/std/math.myc:100` | `fn tadd{M}(a: Ternary{M}, b: Ternary{M}) => Ternary{M}` | — | Empirical/Declared |
| `std.math::tsub` | fn | `lib/std/math.myc:105` | `fn tsub{M}(a: Ternary{M}, b: Ternary{M}) => Ternary{M}` | tsub<M>: balanced-ternary subtraction. Exact on the in-range result; out-of-range is never-silent `Overflow` (G2). | Empirical/Declared |
| `std.math::tmul` | fn | `lib/std/math.myc:110` | `fn tmul{M}(a: Ternary{M}, b: Ternary{M}) => Ternary{M}` | tmul<M>: balanced-ternary multiplication. Exact on the in-range result; out-of-range is never-silent `Overflow` (G2). | Empirical/Declared |
| `std.math::tneg` | fn | `lib/std/math.myc:115` | `fn tneg{M}(a: Ternary{M}) => Ternary{M}` | tneg<M>: balanced-ternary negation (digit-wise sign flip). Always in range, Exact — value(-t) = -value(t) (balanced ternary has no sign asymmetry). | Empirical/Declared |

### std.option

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.option` | nodule | `lib/std/option.myc:8` | `nodule std.option` | Self-hosted Option<A> — the never-silent optional value (Some(A) \| None) plus the core | Empirical/Declared |
| `std.option::Option` | type | `lib/std/option.myc:19` | `type Option[A] = Some(A) \| None` | combinators is_some, is_none, unwrap_or, map, and_then, fold, or_else, flatten. M-715 (E13-1, RFC-0031 §5 D4 Tier-0). Sibling of std.result; follows the lib/std/result.myc composition prototype exactly. Guarantee (per-op, audit-precise): the Some/None constructors and the total Bool discriminators is_some/is_none are Exact (total, RFC-0016 / core spec §3); the value combinators that extract/transform/eliminate (unwrap_or, map, and_then, fold, or_else, flatten) carry a Declared type-level contract. Differential agreement (L1-eval ≡ L0-interp ≡ AOT) is Empirical (trials, std_option.rs). Never-silent (G2): unwrap_or, fold, and or_else take a caller-supplied fallback/eliminator/alternative — no panic, no sentinel; None never silently becomes a value. | Empirical/Declared |
| `std.option::Option::None` | ctor | `lib/std/option.myc:19` | `None` | — | Empirical/Declared |
| `std.option::Option::Some` | ctor | `lib/std/option.myc:19` | `Some(A)` | — | Empirical/Declared |
| `std.option::is_some` | fn | `lib/std/option.myc:22` | `fn is_some[A](o: Option[A]) => Bool` | is_some: True iff the optional holds a value. Total projection (Exact). | Empirical/Declared |
| `std.option::is_none` | fn | `lib/std/option.myc:26` | `fn is_none[A](o: Option[A]) => Bool` | is_none: True iff the optional is empty. Total projection (Exact). | Empirical/Declared |
| `std.option::unwrap_or` | fn | `lib/std/option.myc:31` | `fn unwrap_or[A](o: Option[A], fallback: A) => A` | unwrap_or: the held value, else the caller-supplied fallback. Never-silent (G2): None yields the explicit fallback — no panic, no sentinel. Declared (type-level contract). | Empirical/Declared |
| `std.option::map` | fn | `lib/std/option.myc:36` | `fn map[A, B](o: Option[A], f: A => B) => Option[B]` | map: apply `f` to the held value; None passes through untouched. Why: lift a pure A -> B over an Option without forcing the caller to match — never-silent (None is preserved). Declared. | Empirical/Declared |
| `std.option::and_then` | fn | `lib/std/option.myc:41` | `fn and_then[A, B](o: Option[A], f: A => Option[B]) => Option[B]` | and_then: chain an Option-returning step on Some; None short-circuits. The monadic bind for Option — sequence fallible-by-absence steps without nesting matches. Declared. | Empirical/Declared |
| `std.option::fold` | fn | `lib/std/option.myc:47` | `fn fold[A, B](o: Option[A], on_some: A => B, on_none: B) => B` | fold: eliminate an Option into one B via on_some (a transform) and on_none (a default value). The catamorphism for Option — collapse both cases to a common type, total + never-silent (G2: the None case is an explicit caller-supplied B, never a fabricated one). Declared. | Empirical/Declared |
| `std.option::or_else` | fn | `lib/std/option.myc:53` | `fn or_else[A](o: Option[A], alt: Option[A]) => Option[A]` | or_else: keep a Some; on None, fall back to an alternative Option. Why: chain "try this, else that" over optionals without unwrapping — the alternative is caller-supplied (never-silent, G2). Declared. | Empirical/Declared |
| `std.option::flatten` | fn | `lib/std/option.myc:58` | `fn flatten[A](o: Option[Option[A]]) => Option[A]` | flatten: collapse a nested Option<Option<A>> by one level — an outer None, or an inner None, both yield None; only Some(Some(x)) survives as Some(x). The join for Option. Declared. | Empirical/Declared |

### std.recover

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.recover` | nodule | `lib/std/recover.myc:8` | `nodule std.recover` | Self-hosted declarative recovery bridge — the never-silent driver over a reified | Empirical/Declared |
| `std.recover::Result` | type | `lib/std/recover.myc:101` | `type Result[A, E] = Ok(A) \| Err(E)` | Never-silent floor (C1/I1) carried over intact: `Resolution` has NO drop variant — every path through `handle_classified` yields `Recovered` (honest tag, never upgraded — I2/VR-5) or `Propagated` (the error survives). Budget overruns are explicit (`Exhausted`, I4); an effect with no declared budget cannot run (I5); an undeclared performed effect is an explicit checker error (I3). Guarantee tags (VR-5, carried at the SAME strength as the Rust crate's matrix): Ok pass-through GExact (FR-R3); fallback GDeclared (fixed ceiling); retry success inherits the attempt's own tag; policy/ledger/check ops Exact (no accuracy semantics); differential agreement Empirical (std_recover.rs). ── local mirrors (single-nodule harness — see the substitution notes above) ──────────────────── | Empirical/Declared |
| `std.recover::Result::Err` | ctor | `lib/std/recover.myc:101` | `Err(E)` | — | Empirical/Declared |
| `std.recover::Result::Ok` | ctor | `lib/std/recover.myc:101` | `Ok(A)` | — | Empirical/Declared |
| `std.recover::Option` | type | `lib/std/recover.myc:103` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `std.recover::Option::None` | ctor | `lib/std/recover.myc:103` | `None` | — | Empirical/Declared |
| `std.recover::Option::Some` | ctor | `lib/std/recover.myc:103` | `Some(A)` | — | Empirical/Declared |
| `std.recover::Unit` | type | `lib/std/recover.myc:107` | `type Unit = U` | Unit — the nullary-marker substitute for the missing `() => T` closure-domain form (std.error.myc precedent). | Empirical/Declared |
| `std.recover::Unit::U` | ctor | `lib/std/recover.myc:107` | `U` | — | Empirical/Declared |
| `std.recover::Guarantee` | type | `lib/std/recover.myc:111` | `type Guarantee = GExact \| GProven \| GEmpirical \| GDeclared` | Guarantee — the kernel `GuaranteeStrength` lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` as VALUE-level data, `G`-prefixed (reserved-keyword substitution; std.diag precedent). | Empirical/Declared |
| `std.recover::Guarantee::GDeclared` | ctor | `lib/std/recover.myc:111` | `GDeclared` | — | Empirical/Declared |
| `std.recover::Guarantee::GEmpirical` | ctor | `lib/std/recover.myc:111` | `GEmpirical` | — | Empirical/Declared |
| `std.recover::Guarantee::GExact` | ctor | `lib/std/recover.myc:111` | `GExact` | — | Empirical/Declared |
| `std.recover::Guarantee::GProven` | ctor | `lib/std/recover.myc:111` | `GProven` | — | Empirical/Declared |
| `std.recover::Outcome` | type | `lib/std/recover.myc:117` | `type Outcome[T, E] = OOk(T) \| OErr(E)` | — | Empirical/Declared |
| `std.recover::Outcome::OErr` | ctor | `lib/std/recover.myc:117` | `OErr(E)` | — | Empirical/Declared |
| `std.recover::Outcome::OOk` | ctor | `lib/std/recover.myc:117` | `OOk(T)` | — | Empirical/Declared |
| `std.recover::PolicyWitness` | type | `lib/std/recover.myc:122` | `type PolicyWitness = ByPolicy \| NoPolicy` | PolicyWitness — the honest substitution for `Option<PolicyRef>` (FLAG-recover-1): records WHETHER a policy acted on the outcome (ByPolicy) or none did (NoPolicy). The content-addressed identity itself is kernel-only and is NOT fabricated here (VR-5). | Empirical/Declared |
| `std.recover::PolicyWitness::ByPolicy` | ctor | `lib/std/recover.myc:122` | `ByPolicy` | — | Empirical/Declared |
| `std.recover::PolicyWitness::NoPolicy` | ctor | `lib/std/recover.myc:122` | `NoPolicy` | — | Empirical/Declared |
| `std.recover::Resolution` | type | `lib/std/recover.myc:131` | `type Resolution[T, E] = Recovered(T, Guarantee, PolicyWitness) \| Propagated(E, PolicyWitness, Bool)` | Resolution — the output sum of the driver: `Recovered(value, tag, witness)` or `Propagated(error, witness, cleanup_overrun)`. There is NO `Dropped` variant (I1): a handler cannot express "discard the error" using this sum — never-silent is enforced by the type. The recovered `tag` is never upgraded (I2/VR-5): GDeclared for a fallback, the attempt's own tag for a retry success, GExact for a clean OOk pass-through (FR-R3). `cleanup_overrun` (spec §7-Q4): True means a cleanup_then_propagate cleanup was SKIPPED on budget overrun — recorded, not swallowed; the original error propagates regardless. | Empirical/Declared |
| `std.recover::Resolution::Propagated` | ctor | `lib/std/recover.myc:131` | `Propagated(E, PolicyWitness, Bool)` | — | Empirical/Declared |
| `std.recover::Resolution::Recovered` | ctor | `lib/std/recover.myc:131` | `Recovered(T, Guarantee, PolicyWitness)` | — | Empirical/Declared |
| `std.recover::AttemptOut` | type | `lib/std/recover.myc:136` | `type AttemptOut[T, E] = Attempted(Outcome[T, E], Guarantee)` | AttemptOut — the product substitution for the Rust attempt thunk's `(Outcome<T, E>, GuaranteeStrength)` tuple: the re-attempt's outcome plus that value's honest tag (inherited on retry success, never upgraded — I2/VR-5). | Empirical/Declared |
| `std.recover::AttemptOut::Attempted` | ctor | `lib/std/recover.myc:136` | `Attempted(Outcome[T, E], Guarantee)` | — | Empirical/Declared |
| `std.recover::ClassName` | type | `lib/std/recover.myc:141` | `type ClassName = ClsIoError \| ClsParseError \| ClsTimeout \| ClsFatal` | — | Empirical/Declared |
| `std.recover::ClassName::ClsFatal` | ctor | `lib/std/recover.myc:141` | `ClsFatal` | — | Empirical/Declared |
| `std.recover::ClassName::ClsIoError` | ctor | `lib/std/recover.myc:141` | `ClsIoError` | — | Empirical/Declared |
| `std.recover::ClassName::ClsParseError` | ctor | `lib/std/recover.myc:141` | `ClsParseError` | — | Empirical/Declared |
| `std.recover::ClassName::ClsTimeout` | ctor | `lib/std/recover.myc:141` | `ClsTimeout` | — | Empirical/Declared |
| `std.recover::UnknownClass` | type | `lib/std/recover.myc:145` | `type UnknownClass = UnknownCls(ClassName)` | UnknownClass — the explicit error returned by `resolve`/`on` when a class is not REGISTERED (X1): a configuration error, never a silent fabrication (G2). | Empirical/Declared |
| `std.recover::UnknownClass::UnknownCls` | ctor | `lib/std/recover.myc:145` | `UnknownCls(ClassName)` | — | Empirical/Declared |
| `std.recover::ClassRegistry` | type | `lib/std/recover.myc:150` | `type ClassRegistry = RegNil \| RegCons(ClassName, ClassRegistry)` | ClassRegistry — the append-only runtime registry: a class resolves only if explicitly registered (a runtime SUBSET of the closed vocabulary, so the X1 lookup discipline is preserved, not statically discharged). | Empirical/Declared |
| `std.recover::ClassRegistry::RegCons` | ctor | `lib/std/recover.myc:150` | `RegCons(ClassName, ClassRegistry)` | — | Empirical/Declared |
| `std.recover::ClassRegistry::RegNil` | ctor | `lib/std/recover.myc:150` | `RegNil` | — | Empirical/Declared |
| `std.recover::EffectKind` | type | `lib/std/recover.myc:154` | `type EffectKind = EkRetry \| EkAlloc \| EkIo \| EkCascade \| EkTime` | — | Empirical/Declared |
| `std.recover::EffectKind::EkAlloc` | ctor | `lib/std/recover.myc:154` | `EkAlloc` | — | Empirical/Declared |
| `std.recover::EffectKind::EkCascade` | ctor | `lib/std/recover.myc:154` | `EkCascade` | — | Empirical/Declared |
| `std.recover::EffectKind::EkIo` | ctor | `lib/std/recover.myc:154` | `EkIo` | — | Empirical/Declared |
| `std.recover::EffectKind::EkRetry` | ctor | `lib/std/recover.myc:154` | `EkRetry` | — | Empirical/Declared |
| `std.recover::EffectKind::EkTime` | ctor | `lib/std/recover.myc:154` | `EkTime` | — | Empirical/Declared |
| `std.recover::EffectBudget` | type | `lib/std/recover.myc:160` | `type EffectBudget = Attempts(Binary{8}) \| Depth(Binary{8}) \| AllocBytes(Binary{8}) \| Fuel(Binary{8}) \| Ops(Binary{8})` | EffectBudget — one budget variant per kind, distinct vocabulary (Attempts/Depth/AllocBytes/ Fuel/Ops for Retry/Cascade/Alloc/Time/Io), all enforced by the one `budget_consume` mechanism. `AllocBytes` renames Rust's `Bytes` variant — `Bytes` is a reserved repr-type keyword. Amounts are Binary{8} (see the width substitution note above). | Empirical/Declared |
| `std.recover::EffectBudget::Attempts` | ctor | `lib/std/recover.myc:161` | `Attempts(Binary{8})` | — | Empirical/Declared |
| `std.recover::EffectBudget::Depth` | ctor | `lib/std/recover.myc:162` | `Depth(Binary{8})` | — | Empirical/Declared |
| `std.recover::EffectBudget::AllocBytes` | ctor | `lib/std/recover.myc:163` | `AllocBytes(Binary{8})` | — | Empirical/Declared |
| `std.recover::EffectBudget::Fuel` | ctor | `lib/std/recover.myc:164` | `Fuel(Binary{8})` | — | Empirical/Declared |
| `std.recover::EffectBudget::Ops` | ctor | `lib/std/recover.myc:165` | `Ops(Binary{8})` | — | Empirical/Declared |
| `std.recover::EffectBudgetExhausted` | type | `lib/std/recover.myc:169` | `type EffectBudgetExhausted = Exhausted(EffectKind, Binary{8}, Binary{8})` | EffectBudgetExhausted — the explicit, graceful overrun (I4): `Exhausted(kind, requested, remaining)` — never a hang, stack overflow, or OOM. | Empirical/Declared |
| `std.recover::EffectBudgetExhausted::Exhausted` | ctor | `lib/std/recover.myc:169` | `Exhausted(EffectKind, Binary{8}, Binary{8})` | — | Empirical/Declared |
| `std.recover::Budgets` | type | `lib/std/recover.myc:173` | `type Budgets = BNil \| BEntry(EffectKind, Binary{8}, Budgets)` | Budgets — the budget ledger: one enforcement mechanism over separate named budgets (RFC-0014 §8 resolved). An effect with NO entry cannot consume anything (default tightly scoped, I5). | Empirical/Declared |
| `std.recover::Budgets::BEntry` | ctor | `lib/std/recover.myc:173` | `BEntry(EffectKind, Binary{8}, Budgets)` | — | Empirical/Declared |
| `std.recover::Budgets::BNil` | ctor | `lib/std/recover.myc:173` | `BNil` | — | Empirical/Declared |
| `std.recover::EffectList` | type | `lib/std/recover.myc:177` | `type EffectList = ENil \| ECons(EffectKind, EffectList)` | EffectList — a definition's declared/performed effect SET (§4.5 I3) as a dedup cons-list (BTreeSet substitution). | Empirical/Declared |
| `std.recover::EffectList::ECons` | ctor | `lib/std/recover.myc:177` | `ECons(EffectKind, EffectList)` | — | Empirical/Declared |
| `std.recover::EffectList::ENil` | ctor | `lib/std/recover.myc:177` | `ENil` | — | Empirical/Declared |
| `std.recover::UndeclaredEffect` | type | `lib/std/recover.myc:181` | `type UndeclaredEffect = Undeclared(EffectKind)` | UndeclaredEffect — a performed-but-undeclared effect (I3): an explicit checker error, never silent. | Empirical/Declared |
| `std.recover::UndeclaredEffect::Undeclared` | ctor | `lib/std/recover.myc:181` | `Undeclared(EffectKind)` | — | Empirical/Declared |
| `std.recover::RecoveryAction` | type | `lib/std/recover.myc:191` | `type RecoveryAction[T] = Fallback(T) \| Retry(Binary{8}) \| Escalate(ClassName) \| CleanupThenPropagate(EffectKind)` | — | Empirical/Declared |
| `std.recover::RecoveryAction::Fallback` | ctor | `lib/std/recover.myc:192` | `Fallback(T)` | — | Empirical/Declared |
| `std.recover::RecoveryAction::Retry` | ctor | `lib/std/recover.myc:193` | `Retry(Binary{8})` | — | Empirical/Declared |
| `std.recover::RecoveryAction::Escalate` | ctor | `lib/std/recover.myc:194` | `Escalate(ClassName)` | — | Empirical/Declared |
| `std.recover::RecoveryAction::CleanupThenPropagate` | ctor | `lib/std/recover.myc:195` | `CleanupThenPropagate(EffectKind)` | — | Empirical/Declared |
| `std.recover::Policy` | type | `lib/std/recover.myc:200` | `type Policy[T] = PNil \| PRule(ClassName, RecoveryAction[T], Policy[T])` | Policy — the reified recovery policy: an `on <class> => <action>` association list with replace-on-insert (the BTreeMap substitution). The rule list IS the inspection surface (C3); the content-addressed PolicyRef identity is FLAG-recover-1. | Empirical/Declared |
| `std.recover::Policy::PNil` | ctor | `lib/std/recover.myc:200` | `PNil` | — | Empirical/Declared |
| `std.recover::Policy::PRule` | ctor | `lib/std/recover.myc:200` | `PRule(ClassName, RecoveryAction[T], Policy[T])` | — | Empirical/Declared |
| `std.recover::bin_eq` | fn | `lib/std/recover.myc:204` | `fn bin_eq(a: Binary{8}, b: Binary{8}) => Bool` | — | Empirical/Declared |
| `std.recover::bool_and` | fn | `lib/std/recover.myc:208` | `fn bool_and(a: Bool, b: Bool) => Bool` | bool_and: local total conjunction (std.diag precedent — no cross-leaf import). | Empirical/Declared |
| `std.recover::class_code` | fn | `lib/std/recover.myc:213` | `fn class_code(c: ClassName) => Binary{8}` | class_code: injective code map over the closed ClassName vocabulary (Exact — total, match-defined, checkable by inspection). | Empirical/Declared |
| `std.recover::class_eq` | fn | `lib/std/recover.myc:222` | `fn class_eq(a: ClassName, b: ClassName) => Bool` | class_eq: ADT equality via the injective code map (Exact). | Empirical/Declared |
| `std.recover::effect_code` | fn | `lib/std/recover.myc:226` | `fn effect_code(k: EffectKind) => Binary{8}` | effect_code: injective code map over the closed EffectKind kernel (Exact). | Empirical/Declared |
| `std.recover::effect_eq` | fn | `lib/std/recover.myc:236` | `fn effect_eq(a: EffectKind, b: EffectKind) => Bool` | effect_eq: ADT equality via the injective code map (Exact). | Empirical/Declared |
| `std.recover::from_result` | fn | `lib/std/recover.myc:240` | `fn from_result[T, E](r: Result[T, E]) => Outcome[T, E]` | — | Empirical/Declared |
| `std.recover::into_result` | fn | `lib/std/recover.myc:243` | `fn into_result[T, E](o: Outcome[T, E]) => Result[T, E]` | — | Empirical/Declared |
| `std.recover::is_ok` | fn | `lib/std/recover.myc:246` | `fn is_ok[T, E](o: Outcome[T, E]) => Bool` | — | Empirical/Declared |
| `std.recover::is_err` | fn | `lib/std/recover.myc:249` | `fn is_err[T, E](o: Outcome[T, E]) => Bool` | — | Empirical/Declared |
| `std.recover::is_recovered` | fn | `lib/std/recover.myc:253` | `fn is_recovered[T, E](r: Resolution[T, E]) => Bool` | — | Empirical/Declared |
| `std.recover::is_propagated` | fn | `lib/std/recover.myc:256` | `fn is_propagated[T, E](r: Resolution[T, E]) => Bool` | — | Empirical/Declared |
| `std.recover::resolution_witness` | fn | `lib/std/recover.myc:261` | `fn resolution_witness[T, E](r: Resolution[T, E]) => PolicyWitness` | resolution_witness: the acting-policy witness on either variant (C3 — the presence half of Rust's `Resolution::policy_ref`; the content address is FLAG-recover-1). | Empirical/Declared |
| `std.recover::registry_contains` | fn | `lib/std/recover.myc:265` | `fn registry_contains(reg: ClassRegistry, c: ClassName) => Bool` | — | Empirical/Declared |
| `std.recover::register` | fn | `lib/std/recover.myc:272` | `fn register(reg: ClassRegistry, c: ClassName) => ClassRegistry` | register: idempotent insert (registering the same name twice is a no-op) — Exact. | Empirical/Declared |
| `std.recover::resolve` | fn | `lib/std/recover.myc:277` | `fn resolve(reg: ClassRegistry, c: ClassName) => Result[ClassName, UnknownClass]` | resolve: a class resolves only if registered; unregistered names are an explicit Err(UnknownCls) — never a silent fabrication (X1/G2). Exact. | Empirical/Declared |
| `std.recover::budget_kind` | fn | `lib/std/recover.myc:282` | `fn budget_kind(eb: EffectBudget) => EffectKind` | — | Empirical/Declared |
| `std.recover::budget_amount` | fn | `lib/std/recover.myc:292` | `fn budget_amount(eb: EffectBudget) => Binary{8}` | budget_amount: the budget's scalar ceiling (mirrors EffectBudget::amount) — Exact. | Empirical/Declared |
| `std.recover::budget_set` | fn | `lib/std/recover.myc:297` | `fn budget_set(b: Budgets, eb: EffectBudget) => Budgets` | budget_set: declare (or reset) a budget for its effect kind (mirrors Budgets::set/with) — functional update; Exact. | Empirical/Declared |
| `std.recover::budget_remaining` | fn | `lib/std/recover.myc:307` | `fn budget_remaining(b: Budgets, k: EffectKind) => Option[Binary{8}]` | budget_remaining: the remaining budget for a kind; None if none was declared — Exact. | Empirical/Declared |
| `std.recover::budget_consume` | fn | `lib/std/recover.myc:322` | `fn budget_consume(b: Budgets, k: EffectKind, amount: Binary{8}) => Result[Budgets, EffectBudgetExhausted]` | budget_consume: consume `amount` of `k`'s budget. An overrun — INCLUDING consuming an effect with NO declared budget (I5: remaining 0) — is the explicit, graceful Err(Exhausted(kind, requested, remaining)) (I4); never a hang or a silent stall. On success the NEW ledger (decremented via the Exact sub_u prim) is returned in Ok — the functional form of Rust's `&mut` decrement. Exact (budget arithmetic; no accuracy semantics). | Empirical/Declared |
| `std.recover::effect_member` | fn | `lib/std/recover.myc:342` | `fn effect_member(xs: EffectList, k: EffectKind) => Bool` | — | Empirical/Declared |
| `std.recover::effect_insert` | fn | `lib/std/recover.myc:350` | `fn effect_insert(xs: EffectList, k: EffectKind) => EffectList` | effect_insert: set-insert (no-op when already present) — the dedup that keeps EffectList a set. Exact. | Empirical/Declared |
| `std.recover::check_effects` | fn | `lib/std/recover.myc:356` | `fn check_effects(declared: EffectList, performed: EffectList) => Result[Unit, UndeclaredEffect]` | check_effects: every performed effect must be in the declared set; the FIRST undeclared one is an explicit Err(Undeclared) (I3 — no unknown side effects). This CHECKS that declared effects compose — it never INFERS one. Exact (static check; no accuracy semantics). | Empirical/Declared |
| `std.recover::policy_insert` | fn | `lib/std/recover.myc:370` | `fn policy_insert[T](p: Policy[T], c: ClassName, a: RecoveryAction[T]) => Policy[T]` | — | Empirical/Declared |
| `std.recover::escalate_target` | fn | `lib/std/recover.myc:380` | `fn escalate_target[T](a: RecoveryAction[T]) => Option[ClassName]` | escalate_target: the Escalate action's target class, if any (the X1 validation hook for `on`). | Empirical/Declared |
| `std.recover::on` | fn | `lib/std/recover.myc:393` | `fn on[T](reg: ClassRegistry, p: Policy[T], c: ClassName, a: RecoveryAction[T]) => Result[Policy[T], UnknownClass]` | on: add an `on <class> => <action>` rule, resolving the LHS class AND any Escalate target through the registry (X1 — both are membership-checked, never unvalidated). An unknown class (either side) is an explicit Err(UnknownCls), never a silent fabrication (G2). Returns the UPDATED policy in Ok (functional update — see the substitution note on the dropped prior-action return). Exact (builds a config value; no accuracy semantics). | Empirical/Declared |
| `std.recover::action_for` | fn | `lib/std/recover.myc:412` | `fn action_for[T](p: Policy[T], c: ClassName) => Option[RecoveryAction[T]]` | action_for: the recovery action for a class, if any — None is explicit absence, never a silent fallthrough to a default action (G2). Exact. | Empirical/Declared |
| `std.recover::policy_is_empty` | fn | `lib/std/recover.myc:418` | `fn policy_is_empty[T](p: Policy[T]) => Bool` | — | Empirical/Declared |
| `std.recover::action_effect` | fn | `lib/std/recover.myc:423` | `fn action_effect[T](a: RecoveryAction[T]) => Option[EffectKind]` | action_effect: the effect an action declares, if any (Retry → EkRetry; CleanupThenPropagate → its cleanup effect; Fallback/Escalate are effect-free). | Empirical/Declared |
| `std.recover::policy_effects` | fn | `lib/std/recover.myc:433` | `fn policy_effects[T](p: Policy[T]) => EffectList` | policy_effects: the policy's declared, closed effect set (I3) — the input to check_effects. Exact. | Empirical/Declared |
| `std.recover::retry_loop` | fn | `lib/std/recover.myc:447` | `fn retry_loop[T, E](remaining: Binary{8}, err: E, b: Budgets, attempt: Unit => AttemptOut[T, E]) => Resolution[T, E]` | — | Empirical/Declared |
| `std.recover::apply_action` | fn | `lib/std/recover.myc:472` | `fn apply_action[T, E](a: RecoveryAction[T], err: E, b: Budgets, attempt: Unit => AttemptOut[T, E]) => Resolution[T, E]` | apply_action: apply one matched action — every arm yields Recovered or Propagated (I1). Fallback recovers with the fixed GDeclared ceiling (a substituted value has no checked basis — I2/VR-5). Escalate re-propagates explicitly (the target intent is policy-recorded; the E payload is not re-typed — the honest Rust-first seam). CleanupThenPropagate consumes its declared effect ONCE; an overrun SKIPS the cleanup only and records it (True) — the original error propagates regardless (spec §7-Q4: recorded, not swallowed). | Empirical/Declared |
| `std.recover::handle_classified` | fn | `lib/std/recover.myc:494` | `fn handle_classified[T, E](o: Outcome[T, E], p: Policy[T], b: Budgets, class_of: E => ClassName, attempt: Unit => AttemptOut[T, E]) => Resolution[T, E]` | handle_classified: the spine — map an Outcome through a Policy; ALWAYS yields a Resolution, never a drop (I1 — enforced by the type). OOk passes through Recovered with tag GExact (FR-R3: a clean pass-through is not a fallback substitution — never GDeclared). An OErr with no matching rule propagates UNCHANGED with NoPolicy (the I1 floor). A matched rule is applied per apply_action, with the ByPolicy witness (C3 presence — FLAG-recover-1 for the content address). Guarantee: inherited honest floor (I2/VR-5); total over budgets. | Empirical/Declared |
| `std.recover::recover_classified` | fn | `lib/std/recover.myc:512` | `fn recover_classified[T, E](r: Result[T, E], p: Policy[T], b: Budgets, class_of: E => ClassName, attempt: Unit => AttemptOut[T, E]) => Resolution[T, E]` | recover_classified: the Result bridge — handle_classified with a Result input (the concrete shape that resolves error.md §7-Q1: RecoverOutcome = Recovered \| Propagated, no drop variant, honest inherited tag). | Empirical/Declared |
| `std.recover::Fallibility` | type | `lib/std/recover.myc:527` | `type Fallibility = Total \| FallibleConfig \| FallibleBudget` | — | Empirical/Declared |
| `std.recover::Fallibility::FallibleBudget` | ctor | `lib/std/recover.myc:527` | `FallibleBudget` | — | Empirical/Declared |
| `std.recover::Fallibility::FallibleConfig` | ctor | `lib/std/recover.myc:527` | `FallibleConfig` | — | Empirical/Declared |
| `std.recover::Fallibility::Total` | ctor | `lib/std/recover.myc:527` | `Total` | — | Empirical/Declared |
| `std.recover::Explainable` | type | `lib/std/recover.myc:529` | `type Explainable = PolicyRef \| IsPolicyRef \| NotApplicable` | — | Empirical/Declared |
| `std.recover::Explainable::IsPolicyRef` | ctor | `lib/std/recover.myc:529` | `IsPolicyRef` | — | Empirical/Declared |
| `std.recover::Explainable::NotApplicable` | ctor | `lib/std/recover.myc:529` | `NotApplicable` | — | Empirical/Declared |
| `std.recover::Explainable::PolicyRef` | ctor | `lib/std/recover.myc:529` | `PolicyRef` | — | Empirical/Declared |
| `std.recover::MatrixRow` | type | `lib/std/recover.myc:531` | `type MatrixRow = Row(Bytes, Bytes, Fallibility, Bytes, Bytes, Explainable, Bytes)` | — | Empirical/Declared |
| `std.recover::MatrixRow::Row` | ctor | `lib/std/recover.myc:531` | `Row(Bytes, Bytes, Fallibility, Bytes, Bytes, Explainable, Bytes)` | — | Empirical/Declared |
| `std.recover::row_op` | fn | `lib/std/recover.myc:534` | `fn row_op(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.recover::row_guarantee` | fn | `lib/std/recover.myc:537` | `fn row_guarantee(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.recover::row_fallibility` | fn | `lib/std/recover.myc:540` | `fn row_fallibility(r: MatrixRow) => Fallibility` | — | Empirical/Declared |
| `std.recover::row_error_set` | fn | `lib/std/recover.myc:543` | `fn row_error_set(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.recover::row_effects` | fn | `lib/std/recover.myc:546` | `fn row_effects(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.recover::row_explainable` | fn | `lib/std/recover.myc:549` | `fn row_explainable(r: MatrixRow) => Explainable` | — | Empirical/Declared |
| `std.recover::row_never_silent` | fn | `lib/std/recover.myc:552` | `fn row_never_silent(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.recover::is_total` | fn | `lib/std/recover.myc:556` | `fn is_total(f: Fallibility) => Bool` | — | Empirical/Declared |
| `std.recover::is_fallible_config` | fn | `lib/std/recover.myc:559` | `fn is_fallible_config(f: Fallibility) => Bool` | — | Empirical/Declared |
| `std.recover::is_fallible_budget` | fn | `lib/std/recover.myc:562` | `fn is_fallible_budget(f: Fallibility) => Bool` | — | Empirical/Declared |
| `std.recover::is_expl_policy_ref` | fn | `lib/std/recover.myc:565` | `fn is_expl_policy_ref(x: Explainable) => Bool` | — | Empirical/Declared |
| `std.recover::nonempty` | fn | `lib/std/recover.myc:570` | `fn nonempty(b: Bytes) => Bool` | nonempty: True iff `b` has at least one byte (bytes_len is the Exact prim at Binary{32}; std.diag precedent). | Empirical/Declared |
| `std.recover::row_handle` | fn | `lib/std/recover.myc:574` | `fn row_handle() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_recover_bridge` | fn | `lib/std/recover.myc:585` | `fn row_recover_bridge() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_fallback` | fn | `lib/std/recover.myc:596` | `fn row_fallback() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_retry` | fn | `lib/std/recover.myc:607` | `fn row_retry() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_escalate` | fn | `lib/std/recover.myc:618` | `fn row_escalate() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_cleanup` | fn | `lib/std/recover.myc:629` | `fn row_cleanup() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_on` | fn | `lib/std/recover.myc:640` | `fn row_on() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_policy_ref` | fn | `lib/std/recover.myc:651` | `fn row_policy_ref() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_action_for` | fn | `lib/std/recover.myc:662` | `fn row_action_for() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_consume` | fn | `lib/std/recover.myc:673` | `fn row_consume() => MatrixRow` | — | Empirical/Declared |
| `std.recover::row_check_effects` | fn | `lib/std/recover.myc:684` | `fn row_check_effects() => MatrixRow` | — | Empirical/Declared |
| `std.recover::Vec` | type | `lib/std/recover.myc:696` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.recover::Vec::Cons` | ctor | `lib/std/recover.myc:696` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.recover::Vec::Nil` | ctor | `lib/std/recover.myc:696` | `Nil` | — | Empirical/Declared |
| `std.recover::matrix` | fn | `lib/std/recover.myc:699` | `fn matrix() => Vec[MatrixRow]` | matrix: the full 11-row table, in the same order as guarantee_matrix.rs::MATRIX. | Empirical/Declared |
| `std.recover::matrix_len` | fn | `lib/std/recover.myc:715` | `fn matrix_len(xs: Vec[MatrixRow]) => Binary{8}` | matrix_len: the row count as Binary{8} (11 rows fits trivially). O(n) spine-walk. | Empirical/Declared |
| `std.recover::all_never_silent_nonempty` | fn | `lib/std/recover.myc:722` | `fn all_never_silent_nonempty(xs: Vec[MatrixRow]) => Bool` | — | Empirical/Declared |
| `std.recover::driver_rows_are_total` | fn | `lib/std/recover.myc:729` | `fn driver_rows_are_total() => Bool` | driver_rows_are_total: the handle/recover bridge rows are Total (total over budgets — I1/I4). | Empirical/Declared |
| `std.recover::driver_rows_are_policy_ref_explainable` | fn | `lib/std/recover.myc:733` | `fn driver_rows_are_policy_ref_explainable() => Bool` | driver_rows_are_policy_ref_explainable: both driver rows carry PolicyRef for EXPLAIN (C3). | Empirical/Declared |
| `std.recover::config_ops_fallibility_is_typed` | fn | `lib/std/recover.myc:742` | `fn config_ops_fallibility_is_typed() => Bool` | config_ops_fallibility_is_typed: the typed half of the Rust exact-ops row check — `on` and `check_effects` are FallibleConfig, `consume` is FallibleBudget, `policy_ref`/`action_for` are Total (read off the typed Fallibility column, no substring needed). | Empirical/Declared |

### std.result

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.result` | nodule | `lib/std/result.myc:8` | `nodule std.result` | The first self-hosted generic stdlib nodule — Result<A,E> plus combinators is_ok, is_err, unwrap_or, map, and_then, fold. M-649. Guarantee: Declared (type-level contract; differential agreement Empirical). Never-silent (G2): unwrap_or takes a caller-supplied fallback — no panic, no sentinel. HOF combinators map/and_then/fold are present and executable (RFC-0024 static defunctionalization, M-685/686/687): Declared type-level contract; differential agreement Empirical (three-way L1-eval ≡ L0-interp ≡ AOT, M-688). | Empirical/Declared |
| `std.result::Result` | type | `lib/std/result.myc:10` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `std.result::Result::Err` | ctor | `lib/std/result.myc:10` | `Err(E)` | — | Empirical/Declared |
| `std.result::Result::Ok` | ctor | `lib/std/result.myc:10` | `Ok(A)` | — | Empirical/Declared |
| `std.result::is_ok` | fn | `lib/std/result.myc:12` | `fn is_ok[A, E](r: Result[A, E]) => Bool` | — | Empirical/Declared |
| `std.result::is_err` | fn | `lib/std/result.myc:15` | `fn is_err[A, E](r: Result[A, E]) => Bool` | — | Empirical/Declared |
| `std.result::unwrap_or` | fn | `lib/std/result.myc:18` | `fn unwrap_or[A, E](r: Result[A, E], fallback: A) => A` | — | Empirical/Declared |
| `std.result::map` | fn | `lib/std/result.myc:23` | `fn map[A, B, E](r: Result[A, E], f: A => B) => Result[B, E]` | map: apply `f` to the success value; an Err passes through untouched. Why: lift a pure A -> B transform over a Result without forcing the caller to match — never-silent (Err is preserved). | Empirical/Declared |
| `std.result::and_then` | fn | `lib/std/result.myc:28` | `fn and_then[A, B, E](r: Result[A, E], f: A => Result[B, E]) => Result[B, E]` | and_then: chain a Result-returning step on success; an Err short-circuits. Why: sequence fallible steps without nesting matches — the monadic bind for Result. | Empirical/Declared |
| `std.result::fold` | fn | `lib/std/result.myc:33` | `fn fold[A, E, B](r: Result[A, E], on_ok: A => B, on_err: E => B) => B` | fold: eliminate a Result into one B via on_ok/on_err — the catamorphism (two single-arg fns, so no multi-arg arrow needed). Why: collapse both cases to a common type, total + never-silent. | Empirical/Declared |
| `std.result::map_err` | fn | `lib/std/result.myc:39` | `fn map_err[A, E, F](r: Result[A, E], f: E => F) => Result[A, F]` | map_err: transform the error; an Ok passes through untouched. The mirror of `map` on the Err side — re-type or enrich an error without disturbing the success path (never-silent: Ok is preserved). Declared. | Empirical/Declared |
| `std.result::or_else` | fn | `lib/std/result.myc:45` | `fn or_else[A, E](r: Result[A, E], f: E => Result[A, E]) => Result[A, E]` | or_else: keep an Ok; on Err, run a recovery step that may itself fail. The Err-side bind — the dual of `and_then` (which binds on Ok). Why: sequence fallback/recovery without nesting matches, the error stays explicit (never-silent, G2). Declared. | Empirical/Declared |

### std.select

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.select` | nodule | `lib/std/select.myc:8` | `nodule std.select` | Self-hosted structural selection/decision surface — a reified, total decision table | Empirical/Declared |
| `std.select::Result` | type | `lib/std/select.myc:105` | `type Result[A, E] = Ok(A) \| Err(E)` | Honesty tags (VR-5 — carried at the SAME strength as crates/mycelium-std-select, never upgraded in translation): - Every ported op is `Exact` (C2): the policy is a total predicate over \*exact\* integer metadata — nothing probabilistic, learned, or estimated. The `Exact` tag covers the \*selection decision\*, not downstream op accuracy (the crate's own C2 wording). - The guarantee-matrix row DATA below is `Declared` (asserted, matching the Rust source's own structural tests — diag-port precedent). - Three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT) AND the live Rust-oracle EXPLAIN parity are `Empirical` (trials, std_select.rs — the M-928 DoD row). ── local mirrors (single-nodule harness — see the substitution notes above) ──────────────────── | Empirical/Declared |
| `std.select::Result::Err` | ctor | `lib/std/select.myc:105` | `Err(E)` | — | Empirical/Declared |
| `std.select::Result::Ok` | ctor | `lib/std/select.myc:105` | `Ok(A)` | — | Empirical/Declared |
| `std.select::Option` | type | `lib/std/select.myc:107` | `type Option[OptA] = Some(OptA) \| None` | — | Empirical/Declared |
| `std.select::Option::None` | ctor | `lib/std/select.myc:107` | `None` | — | Empirical/Declared |
| `std.select::Option::Some` | ctor | `lib/std/select.myc:107` | `Some(OptA)` | — | Empirical/Declared |
| `std.select::Guarantee` | type | `lib/std/select.myc:111` | `type Guarantee = GExact \| GProven \| GEmpirical \| GDeclared` | Guarantee — the kernel `GuaranteeStrength` lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` as VALUE-level data, G-prefixed (reserved-keyword substitution; std.diag precedent). | Empirical/Declared |
| `std.select::Guarantee::GDeclared` | ctor | `lib/std/select.myc:111` | `GDeclared` | — | Empirical/Declared |
| `std.select::Guarantee::GEmpirical` | ctor | `lib/std/select.myc:111` | `GEmpirical` | — | Empirical/Declared |
| `std.select::Guarantee::GExact` | ctor | `lib/std/select.myc:111` | `GExact` | — | Empirical/Declared |
| `std.select::Guarantee::GProven` | ctor | `lib/std/select.myc:111` | `GProven` | — | Empirical/Declared |
| `std.select::grank` | fn | `lib/std/select.myc:115` | `fn grank(g: Guarantee) => Binary{8}` | grank: the kernel `GuaranteeStrength::rank` verbatim (Exact = 0 strongest … Declared = 3 weakest; crates/mycelium-core/src/guarantee.rs). Total, Exact. | Empirical/Declared |
| `std.select::ScalarKind` | type | `lib/std/select.myc:124` | `type ScalarKind = SkF16 \| SkBf16 \| SkF32 \| SkF64` | — | Empirical/Declared |
| `std.select::ScalarKind::SkBf16` | ctor | `lib/std/select.myc:124` | `SkBf16` | — | Empirical/Declared |
| `std.select::ScalarKind::SkF16` | ctor | `lib/std/select.myc:124` | `SkF16` | — | Empirical/Declared |
| `std.select::ScalarKind::SkF32` | ctor | `lib/std/select.myc:124` | `SkF32` | — | Empirical/Declared |
| `std.select::ScalarKind::SkF64` | ctor | `lib/std/select.myc:124` | `SkF64` | — | Empirical/Declared |
| `std.select::dtype_bits` | fn | `lib/std/select.myc:128` | `fn dtype_bits(dt: ScalarKind) => Binary{16}` | dtype_bits: bits per Dense element (kernel `dtype_bits` verbatim: F16/Bf16 = 16, F32 = 32, F64 = 64). Total, Exact. | Empirical/Declared |
| `std.select::sk_eq` | fn | `lib/std/select.myc:137` | `fn sk_eq(a: ScalarKind, b: ScalarKind) => Bool` | sk_eq: ScalarKind equality (typed ADT, never stringly). Total, Exact (finite domain). | Empirical/Declared |
| `std.select::ReprDesc` | type | `lib/std/select.myc:152` | `type ReprDesc = RBinary(Binary{16}) \| RTernary(Binary{16}) \| RDense(Binary{16}, ScalarKind) \| RVsaDense(Binary{16}) \| RVsaSparse(Binary{16}) \| RSeq(ReprDesc, Binary{16}) \| RBytes` | — | Empirical/Declared |
| `std.select::ReprDesc::RBinary` | ctor | `lib/std/select.myc:153` | `RBinary(Binary{16})` | — | Empirical/Declared |
| `std.select::ReprDesc::RTernary` | ctor | `lib/std/select.myc:154` | `RTernary(Binary{16})` | — | Empirical/Declared |
| `std.select::ReprDesc::RDense` | ctor | `lib/std/select.myc:155` | `RDense(Binary{16}, ScalarKind)` | — | Empirical/Declared |
| `std.select::ReprDesc::RVsaDense` | ctor | `lib/std/select.myc:156` | `RVsaDense(Binary{16})` | — | Empirical/Declared |
| `std.select::ReprDesc::RVsaSparse` | ctor | `lib/std/select.myc:157` | `RVsaSparse(Binary{16})` | — | Empirical/Declared |
| `std.select::ReprDesc::RSeq` | ctor | `lib/std/select.myc:158` | `RSeq(ReprDesc, Binary{16})` | — | Empirical/Declared |
| `std.select::ReprDesc::RBytes` | ctor | `lib/std/select.myc:159` | `RBytes` | — | Empirical/Declared |
| `std.select::Kind` | type | `lib/std/select.myc:162` | `type Kind = KBinary \| KTernary \| KDense \| KVsa \| KSeq \| KBytes` | — | Empirical/Declared |
| `std.select::Kind::KBinary` | ctor | `lib/std/select.myc:162` | `KBinary` | — | Empirical/Declared |
| `std.select::Kind::KBytes` | ctor | `lib/std/select.myc:162` | `KBytes` | — | Empirical/Declared |
| `std.select::Kind::KDense` | ctor | `lib/std/select.myc:162` | `KDense` | — | Empirical/Declared |
| `std.select::Kind::KSeq` | ctor | `lib/std/select.myc:162` | `KSeq` | — | Empirical/Declared |
| `std.select::Kind::KTernary` | ctor | `lib/std/select.myc:162` | `KTernary` | — | Empirical/Declared |
| `std.select::Kind::KVsa` | ctor | `lib/std/select.myc:162` | `KVsa` | — | Empirical/Declared |
| `std.select::kind_of` | fn | `lib/std/select.myc:165` | `fn kind_of(r: ReprDesc) => Kind` | kind_of: kernel `kind_of` verbatim — the paradigm kind of a source descriptor. Total, Exact. | Empirical/Declared |
| `std.select::kind_eq` | fn | `lib/std/select.myc:177` | `fn kind_eq(a: Kind, b: Kind) => Bool` | kind_eq: Kind equality (typed ADT). Total, Exact (finite domain). | Empirical/Declared |
| `std.select::repr_bits` | fn | `lib/std/select.myc:195` | `fn repr_bits(r: ReprDesc) => Binary{16}` | — | Empirical/Declared |
| `std.select::Cand` | type | `lib/std/select.myc:207` | `type Cand = CRepr(ReprDesc)` | — | Empirical/Declared |
| `std.select::Cand::CRepr` | ctor | `lib/std/select.myc:207` | `CRepr(ReprDesc)` | — | Empirical/Declared |
| `std.select::cand_bits` | fn | `lib/std/select.myc:210` | `fn cand_bits(c: Cand) => Binary{16}` | cand_bits: the deterministic cost of a candidate in declared storage bits. Total, Exact. | Empirical/Declared |
| `std.select::Inputs` | type | `lib/std/select.myc:215` | `type Inputs = SelIn(ReprDesc, Guarantee)` | — | Empirical/Declared |
| `std.select::Inputs::SelIn` | ctor | `lib/std/select.myc:215` | `SelIn(ReprDesc, Guarantee)` | — | Empirical/Declared |
| `std.select::in_src` | fn | `lib/std/select.myc:218` | `fn in_src(i: Inputs) => ReprDesc` | in_src / in_guarantee: field accessors (no field-projection surface in `.myc` v0). Total, Exact. | Empirical/Declared |
| `std.select::in_guarantee` | fn | `lib/std/select.myc:221` | `fn in_guarantee(i: Inputs) => Guarantee` | — | Empirical/Declared |
| `std.select::Predicate` | type | `lib/std/select.myc:227` | `type Predicate = PAlways \| PSrcKindIs(Kind) \| PDtypeIs(ScalarKind) \| PGuaranteeAtLeast(Guarantee) \| PDeclaredSparse \| PAnd(Predicate, Predicate) \| POr(Predicate, Predicate) \| PNot(Predicate)` | — | Empirical/Declared |
| `std.select::Predicate::PAlways` | ctor | `lib/std/select.myc:228` | `PAlways` | — | Empirical/Declared |
| `std.select::Predicate::PSrcKindIs` | ctor | `lib/std/select.myc:229` | `PSrcKindIs(Kind)` | — | Empirical/Declared |
| `std.select::Predicate::PDtypeIs` | ctor | `lib/std/select.myc:230` | `PDtypeIs(ScalarKind)` | — | Empirical/Declared |
| `std.select::Predicate::PGuaranteeAtLeast` | ctor | `lib/std/select.myc:231` | `PGuaranteeAtLeast(Guarantee)` | — | Empirical/Declared |
| `std.select::Predicate::PDeclaredSparse` | ctor | `lib/std/select.myc:232` | `PDeclaredSparse` | — | Empirical/Declared |
| `std.select::Predicate::PAnd` | ctor | `lib/std/select.myc:233` | `PAnd(Predicate, Predicate)` | — | Empirical/Declared |
| `std.select::Predicate::POr` | ctor | `lib/std/select.myc:234` | `POr(Predicate, Predicate)` | — | Empirical/Declared |
| `std.select::Predicate::PNot` | ctor | `lib/std/select.myc:235` | `PNot(Predicate)` | — | Empirical/Declared |
| `std.select::rank_lte` | fn | `lib/std/select.myc:239` | `fn rank_lte(g: Guarantee, req: Guarantee) => Bool` | rank_lte: "the disclosed guarantee is at least this strong" — kernel: g.rank() <= req.rank() (lattice rank ≤; lower rank = stronger). Total, Exact. | Empirical/Declared |
| `std.select::eval_pred` | fn | `lib/std/select.myc:244` | `fn eval_pred(p: Predicate, i: Inputs) => Bool` | eval_pred: kernel `Predicate::eval` — total: every predicate yields a Bool on every input, no partiality, no side effects. Exact (a total predicate over exact metadata — C2). | Empirical/Declared |
| `std.select::Action` | type | `lib/std/select.myc:257` | `type Action = AChoose(Binary{8}) \| ACheapest` | — | Empirical/Declared |
| `std.select::Action::ACheapest` | ctor | `lib/std/select.myc:257` | `ACheapest` | — | Empirical/Declared |
| `std.select::Action::AChoose` | ctor | `lib/std/select.myc:257` | `AChoose(Binary{8})` | — | Empirical/Declared |
| `std.select::Rule` | type | `lib/std/select.myc:259` | `type Rule = When(Predicate, Action)` | — | Empirical/Declared |
| `std.select::Rule::When` | ctor | `lib/std/select.myc:259` | `When(Predicate, Action)` | — | Empirical/Declared |
| `std.select::CandList` | type | `lib/std/select.myc:262` | `type CandList = CLNil \| CLCons(Cand, CandList)` | — | Empirical/Declared |
| `std.select::CandList::CLCons` | ctor | `lib/std/select.myc:262` | `CLCons(Cand, CandList)` | — | Empirical/Declared |
| `std.select::CandList::CLNil` | ctor | `lib/std/select.myc:262` | `CLNil` | — | Empirical/Declared |
| `std.select::RuleList` | type | `lib/std/select.myc:264` | `type RuleList = RLNil \| RLCons(Rule, RuleList)` | — | Empirical/Declared |
| `std.select::RuleList::RLCons` | ctor | `lib/std/select.myc:264` | `RLCons(Rule, RuleList)` | — | Empirical/Declared |
| `std.select::RuleList::RLNil` | ctor | `lib/std/select.myc:264` | `RLNil` | — | Empirical/Declared |
| `std.select::cand_len` | fn | `lib/std/select.myc:268` | `fn cand_len(cs: CandList) => Binary{8}` | cand_len: the candidate count. O(n) spine walk; > 255 candidates is add_u's explicit never-silent Overflow refusal (G2), never a wrap. Exact on the in-range domain. | Empirical/Declared |
| `std.select::cand_at` | fn | `lib/std/select.myc:273` | `fn cand_at(cs: CandList, i: Binary{8}) => Option[Cand]` | cand_at: the candidate at index `i`, or None if out of range (never-silent, G2). Declared (type-level contract). | Empirical/Declared |
| `std.select::SelectionPolicy` | type | `lib/std/select.myc:287` | `type SelectionPolicy = SPol(Bytes, CandList, RuleList, Binary{8})` | — | Empirical/Declared |
| `std.select::SelectionPolicy::SPol` | ctor | `lib/std/select.myc:287` | `SPol(Bytes, CandList, RuleList, Binary{8})` | — | Empirical/Declared |
| `std.select::pol_name` | fn | `lib/std/select.myc:290` | `fn pol_name(p: SelectionPolicy) => Bytes` | pol_name / pol_cands / pol_rules / pol_dflt: field accessors. Total, Exact. | Empirical/Declared |
| `std.select::pol_cands` | fn | `lib/std/select.myc:293` | `fn pol_cands(p: SelectionPolicy) => CandList` | — | Empirical/Declared |
| `std.select::pol_rules` | fn | `lib/std/select.myc:296` | `fn pol_rules(p: SelectionPolicy) => RuleList` | — | Empirical/Declared |
| `std.select::pol_dflt` | fn | `lib/std/select.myc:299` | `fn pol_dflt(p: SelectionPolicy) => Binary{8}` | — | Empirical/Declared |
| `std.select::PolicyError` | type | `lib/std/select.myc:304` | `type PolicyError = NoCandidates \| IndexOutOfRange(Binary{8})` | — | Empirical/Declared |
| `std.select::PolicyError::IndexOutOfRange` | ctor | `lib/std/select.myc:304` | `IndexOutOfRange(Binary{8})` | — | Empirical/Declared |
| `std.select::PolicyError::NoCandidates` | ctor | `lib/std/select.myc:304` | `NoCandidates` | — | Empirical/Declared |
| `std.select::first_bad_choose` | fn | `lib/std/select.myc:308` | `fn first_bad_choose(rs: RuleList, n: Binary{8}) => Option[Binary{8}]` | first_bad_choose: the first rule Choose index outside `n`, or None when all are in range. Total, Exact (structural recursion on the finite table). | Empirical/Declared |
| `std.select::build` | fn | `lib/std/select.myc:323` | `fn build(name: Bytes, cands: CandList, rules: RuleList, dflt: Binary{8}) => Result[SelectionPolicy, PolicyError]` | build: validate totality up front — at least one candidate, the default arm and every Choose(i) in range (kernel `SelectionPolicy::new` minus the FLAG-select-2 float checks). Guarantee: Exact — either the policy is accepted (total, well-formed) or refused with an explicit PolicyError; there is no silent completion of a non-total table (C1). | Empirical/Declared |
| `std.select::CandidateCost` | type | `lib/std/select.myc:343` | `type CandidateCost = CCost(Cand, Binary{16})` | — | Empirical/Declared |
| `std.select::CandidateCost::CCost` | ctor | `lib/std/select.myc:343` | `CCost(Cand, Binary{16})` | — | Empirical/Declared |
| `std.select::CostList` | type | `lib/std/select.myc:345` | `type CostList = KNil \| KCons(CandidateCost, CostList)` | — | Empirical/Declared |
| `std.select::CostList::KCons` | ctor | `lib/std/select.myc:345` | `KCons(CandidateCost, CostList)` | — | Empirical/Declared |
| `std.select::CostList::KNil` | ctor | `lib/std/select.myc:345` | `KNil` | — | Empirical/Declared |
| `std.select::line_cand` | fn | `lib/std/select.myc:348` | `fn line_cand(l: CandidateCost) => Cand` | line_cand / line_cost: cost-line accessors. Total, Exact. | Empirical/Declared |
| `std.select::line_cost` | fn | `lib/std/select.myc:351` | `fn line_cost(l: CandidateCost) => Binary{16}` | — | Empirical/Declared |
| `std.select::costs_len` | fn | `lib/std/select.myc:355` | `fn costs_len(ks: CostList) => Binary{8}` | costs_len: the ranking length. Exact on the in-range domain (add_u refusal past 255 — G2). | Empirical/Declared |
| `std.select::cost_at` | fn | `lib/std/select.myc:359` | `fn cost_at(ks: CostList, i: Binary{8}) => Option[CandidateCost]` | cost_at: the cost line at index `i`, or None if out of range (never-silent, G2). Declared. | Empirical/Declared |
| `std.select::costs_of` | fn | `lib/std/select.myc:370` | `fn costs_of(cs: CandList) => CostList` | costs_of: every candidate's cost line, in candidate order. Total, Exact. | Empirical/Declared |
| `std.select::Explanation` | type | `lib/std/select.myc:379` | `type Explanation = Expl(Bytes, Inputs, CostList, Option[Binary{8}], Binary{8}, Cand, Bool)` | Explanation — emitted on EVERY selection (C3: the module's reason to exist): policy_name (the deciding policy by name; its content address is FLAG-select-1), the inputs considered, the cost of each candidate, which rule matched (None = the default arm or an override), the chosen index + candidate, and the override state. Deterministic and re-derivable from (policy, inputs) alone. Expl(policy_name, inputs, costs, matched_rule, chosen_index, chosen, overridden) | Empirical/Declared |
| `std.select::Explanation::Expl` | ctor | `lib/std/select.myc:379` | `Expl(Bytes, Inputs, CostList, Option[Binary{8}], Binary{8}, Cand, Bool)` | — | Empirical/Declared |
| `std.select::expl_policy_name` | fn | `lib/std/select.myc:382` | `fn expl_policy_name(e: Explanation) => Bytes` | Explanation field accessors. Total, Exact. | Empirical/Declared |
| `std.select::expl_inputs` | fn | `lib/std/select.myc:385` | `fn expl_inputs(e: Explanation) => Inputs` | — | Empirical/Declared |
| `std.select::expl_costs` | fn | `lib/std/select.myc:388` | `fn expl_costs(e: Explanation) => CostList` | — | Empirical/Declared |
| `std.select::expl_matched_rule` | fn | `lib/std/select.myc:391` | `fn expl_matched_rule(e: Explanation) => Option[Binary{8}]` | — | Empirical/Declared |
| `std.select::expl_chosen_index` | fn | `lib/std/select.myc:394` | `fn expl_chosen_index(e: Explanation) => Binary{8}` | — | Empirical/Declared |
| `std.select::expl_chosen` | fn | `lib/std/select.myc:397` | `fn expl_chosen(e: Explanation) => Cand` | — | Empirical/Declared |
| `std.select::expl_overridden` | fn | `lib/std/select.myc:400` | `fn expl_overridden(e: Explanation) => Bool` | — | Empirical/Declared |
| `std.select::Selected` | type | `lib/std/select.myc:404` | `type Selected = Chosen(Cand, Explanation)` | Selected — the (choice, explanation) pair every selection returns (no tuple surface form). | Empirical/Declared |
| `std.select::Selected::Chosen` | ctor | `lib/std/select.myc:404` | `Chosen(Cand, Explanation)` | — | Empirical/Declared |
| `std.select::sel_cand` | fn | `lib/std/select.myc:406` | `fn sel_cand(s: Selected) => Cand` | — | Empirical/Declared |
| `std.select::sel_expl` | fn | `lib/std/select.myc:409` | `fn sel_expl(s: Selected) => Explanation` | — | Empirical/Declared |
| `std.select::SelectError` | type | `lib/std/select.myc:416` | `type SelectError = OverrideOutOfRange(Binary{8}, Binary{8}) \| DanglingChoice(Binary{8})` | — | Empirical/Declared |
| `std.select::SelectError::DanglingChoice` | ctor | `lib/std/select.myc:416` | `DanglingChoice(Binary{8})` | — | Empirical/Declared |
| `std.select::SelectError::OverrideOutOfRange` | ctor | `lib/std/select.myc:416` | `OverrideOutOfRange(Binary{8}, Binary{8})` | — | Empirical/Declared |
| `std.select::MatchHit` | type | `lib/std/select.myc:419` | `type MatchHit = Hit(Binary{8}, Action)` | MatchHit — the first matching rule: its table index and its action. | Empirical/Declared |
| `std.select::MatchHit::Hit` | ctor | `lib/std/select.myc:419` | `Hit(Binary{8}, Action)` | — | Empirical/Declared |
| `std.select::find_rule` | fn | `lib/std/select.myc:423` | `fn find_rule(rs: RuleList, i: Inputs, idx: Binary{8}) => Option[MatchHit]` | find_rule: the first rule whose guard holds (table order — fixed declared precedence, RFC-0005 §2.3), or None when no rule matches (the default arm decides). Total, Exact. | Empirical/Declared |
| `std.select::cheapest_from` | fn | `lib/std/select.myc:437` | `fn cheapest_from(ks: CostList, best_i: Binary{8}, best_c: Binary{16}, i: Binary{8}) => Binary{8}` | cheapest_from: index of the minimum-cost line over `ks`, seeded with the best so far; ties break to the LOWEST index (strict `lt` — deterministic, kernel `cheapest` verbatim). Total, Exact. | Empirical/Declared |
| `std.select::cheapest_of` | fn | `lib/std/select.myc:449` | `fn cheapest_of(cs: CandList) => Binary{8}` | cheapest_of: index of the minimum-cost candidate. The CLNil arm yields index 0, which every downstream path refuses explicitly (`cand_at` on an empty list is None → DanglingChoice) — no silent path survives an empty candidate set (G2; a built policy is non-empty by `build`). | Empirical/Declared |
| `std.select::finish` | fn | `lib/std/select.myc:458` | `fn finish(nm: Bytes, i: Inputs, cands: CandList, mr: Option[Binary{8}], ci: Binary{8}, ov: Bool) => Result[Selected, SelectError]` | finish: assemble the mandatory Explanation for a decided index — there is NO code path that returns a choice without one (C3). A dangling index is the explicit DanglingChoice refusal (FLAG-select-6), never a fabricated candidate. | Empirical/Declared |
| `std.select::run_select` | fn | `lib/std/select.myc:477` | `fn run_select(pol: SelectionPolicy, i: Inputs, forced: Option[Binary{8}]) => Result[Selected, SelectError]` | run_select: the ONE selection mechanism (RFC-0005 §2/§4; kernel `select` verbatim over the ported surface): honor a first-class forced override (recorded in the EXPLAIN trace, matched rule None) or evaluate the decision table — first matching rule wins; no match → the mandatory default arm. Guarantee: Exact — same (policy, inputs, forced) → same (choice, explanation), deterministically. Errors are explicit: OverrideOutOfRange (never a snap to the nearest legal choice — C1), DanglingChoice (FLAG-select-6). | Empirical/Declared |
| `std.select::select` | fn | `lib/std/select.myc:509` | `fn select(pol: SelectionPolicy, i: Inputs) => Result[Selected, SelectError]` | select: evaluate the decision table and return the chosen candidate WITH its mandatory Explanation — there is no code path that returns a choice without one (C3). Guarantee: Exact (the selection decision over exact metadata — C2). | Empirical/Declared |
| `std.select::explain` | fn | `lib/std/select.myc:517` | `fn explain(pol: SelectionPolicy, i: Inputs) => Result[Explanation, SelectError]` | explain: derive the mandatory Explanation for a (policy, inputs) pair without consuming the choice — deterministic and re-derivable from (policy, inputs) alone (C3/C4). Guarantee: Exact. Fallibility substitution (FLAG-select-6): Result-typed here where the Rust `explain` is total — the totality rests on a private-field invariant `.myc` v0 cannot state; for any policy out of `build` the Err arms are unreachable. | Empirical/Declared |
| `std.select::select_with_override` | fn | `lib/std/select.myc:525` | `fn select_with_override(pol: SelectionPolicy, i: Inputs, forced_index: Binary{8}) => Result[Selected, SelectError]` | select_with_override: a first-class deterministic override — force a candidate by index and record the override state IN the Explanation (overridden = True; matched_rule = None), so the overridden selection remains fully inspectable (RFC-0005 §2.4; C1/C3). An out-of-range forced index is the explicit OverrideOutOfRange refusal, never a snap to the nearest legal choice. Guarantee: Exact. | Empirical/Declared |
| `std.select::ExplainAble` | type | `lib/std/select.myc:537` | `type ExplainAble = ExplainYes \| ExplainNotApplicable` | — | Empirical/Declared |
| `std.select::ExplainAble::ExplainNotApplicable` | ctor | `lib/std/select.myc:537` | `ExplainNotApplicable` | — | Empirical/Declared |
| `std.select::ExplainAble::ExplainYes` | ctor | `lib/std/select.myc:537` | `ExplainYes` | — | Empirical/Declared |
| `std.select::GRow` | type | `lib/std/select.myc:540` | `type GRow = GR(Bytes, Guarantee, Bool, Bytes, ExplainAble)` | GRow(op, tag, fallible, effects, explain_able) — mirrors mycelium-std-select::GuaranteeRow. | Empirical/Declared |
| `std.select::GRow::GR` | ctor | `lib/std/select.myc:540` | `GR(Bytes, Guarantee, Bool, Bytes, ExplainAble)` | — | Empirical/Declared |
| `std.select::GRowList` | type | `lib/std/select.myc:542` | `type GRowList = GLNil \| GLCons(GRow, GRowList)` | — | Empirical/Declared |
| `std.select::GRowList::GLCons` | ctor | `lib/std/select.myc:542` | `GLCons(GRow, GRowList)` | — | Empirical/Declared |
| `std.select::GRowList::GLNil` | ctor | `lib/std/select.myc:542` | `GLNil` | — | Empirical/Declared |
| `std.select::grow_op` | fn | `lib/std/select.myc:545` | `fn grow_op(r: GRow) => Bytes` | GRow field accessors. Total, Exact. | Empirical/Declared |
| `std.select::grow_tag` | fn | `lib/std/select.myc:548` | `fn grow_tag(r: GRow) => Guarantee` | — | Empirical/Declared |
| `std.select::grow_fallible` | fn | `lib/std/select.myc:551` | `fn grow_fallible(r: GRow) => Bool` | — | Empirical/Declared |
| `std.select::grow_effects` | fn | `lib/std/select.myc:554` | `fn grow_effects(r: GRow) => Bytes` | — | Empirical/Declared |
| `std.select::grow_explain_able` | fn | `lib/std/select.myc:557` | `fn grow_explain_able(r: GRow) => ExplainAble` | — | Empirical/Declared |
| `std.select::matrix_len` | fn | `lib/std/select.myc:561` | `fn matrix_len(rs: GRowList) => Binary{8}` | matrix_len: the row count. Exact on the in-range domain. | Empirical/Declared |
| `std.select::matrix_all_exact` | fn | `lib/std/select.myc:566` | `fn matrix_all_exact(rs: GRowList) => Bool` | matrix_all_exact: True iff every row carries the GExact tag (the VR-5 assertion the Rust test suite makes over GUARANTEE_MATRIX). Total, Exact. | Empirical/Declared |
| `std.select::guarantee_matrix` | fn | `lib/std/select.myc:574` | `fn guarantee_matrix() => GRowList` | guarantee_matrix: the loaded matrix — 4 rows (the ported ops), all GExact, EXPLAIN-able = yes for every selection op; `build` constructs a policy and is not itself a selection. | Empirical/Declared |

### std.spores

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.spores` | nodule | `lib/std/spore.myc:8` | `nodule std.spores` | Self-hosted deployable / reconstruction-manifest surface — the structural half of | Empirical/Declared |
| `std.spores::Result` | type | `lib/std/spore.myc:117` | `type Result[A, E] = Ok(A) \| Err(E)` | Never-silent floor (C1/G2) carried over intact: a hash mismatch is the explicit, named HashMismatch(expected, found) — both hashes carried (G11) — never a silent accept; a missing/ambiguous deploy input is the named DeployError; an over-strength resonator manifest is unrepresentable through `manifest_new`/`regrowth_new` (the FR-C2 ceiling, VR-5). Guarantee tags (VR-5, carried at the SAME strength as the Rust crate's matrix): identity/verify/ manifest/validate/display ops Exact (deterministic, no accuracy semantics); germinate Empirical (the native deploy path is not proven end-to-end — the oracle's own tag, never upgraded by translation); the no-opaque-lowering check Declared (structural assertion, MLIR pending); matrix row DATA Declared (hand-transcribed, live-oracle-checked in std_spore.rs); differential agreement Empirical (std_spore.rs). ── local mirrors (single-nodule harness — see the substitution notes above) ──────────────────── | Empirical/Declared |
| `std.spores::Result::Err` | ctor | `lib/std/spore.myc:117` | `Err(E)` | — | Empirical/Declared |
| `std.spores::Result::Ok` | ctor | `lib/std/spore.myc:117` | `Ok(A)` | — | Empirical/Declared |
| `std.spores::Option` | type | `lib/std/spore.myc:119` | `type Option[A] = Some(A) \| None` | — | Empirical/Declared |
| `std.spores::Option::None` | ctor | `lib/std/spore.myc:119` | `None` | — | Empirical/Declared |
| `std.spores::Option::Some` | ctor | `lib/std/spore.myc:119` | `Some(A)` | — | Empirical/Declared |
| `std.spores::Vec` | type | `lib/std/spore.myc:121` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.spores::Vec::Cons` | ctor | `lib/std/spore.myc:121` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.spores::Vec::Nil` | ctor | `lib/std/spore.myc:121` | `Nil` | — | Empirical/Declared |
| `std.spores::Unit` | type | `lib/std/spore.myc:124` | `type Unit = U` | Unit — the nullary marker for `verify`'s `Ok(())` (std.error/std.recover precedent). | Empirical/Declared |
| `std.spores::Unit::U` | ctor | `lib/std/spore.myc:124` | `U` | — | Empirical/Declared |
| `std.spores::Guarantee` | type | `lib/std/spore.myc:128` | `type Guarantee = GExact \| GProven \| GEmpirical \| GDeclared` | Guarantee — the kernel `GuaranteeStrength` lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` as VALUE-level data, `G`-prefixed (reserved-keyword substitution; std.diag precedent). | Empirical/Declared |
| `std.spores::Guarantee::GDeclared` | ctor | `lib/std/spore.myc:128` | `GDeclared` | — | Empirical/Declared |
| `std.spores::Guarantee::GEmpirical` | ctor | `lib/std/spore.myc:128` | `GEmpirical` | — | Empirical/Declared |
| `std.spores::Guarantee::GExact` | ctor | `lib/std/spore.myc:128` | `GExact` | — | Empirical/Declared |
| `std.spores::Guarantee::GProven` | ctor | `lib/std/spore.myc:128` | `GProven` | — | Empirical/Declared |
| `std.spores::bool_and` | fn | `lib/std/spore.myc:131` | `fn bool_and(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `std.spores::bool_not` | fn | `lib/std/spore.myc:134` | `fn bool_not(a: Bool) => Bool` | — | Empirical/Declared |
| `std.spores::bool_text` | fn | `lib/std/spore.myc:138` | `fn bool_text(b: Bool) => Bytes` | bool_text: the Rust `Display for bool` rendering ("true"/"false") — Exact. | Empirical/Declared |
| `std.spores::cat` | fn | `lib/std/spore.myc:142` | `fn cat(a: Bytes, b: Bytes) => Bytes` | cat: byte-string concatenation (the Exact, total bytes_concat prim — RFC-0032 D4). | Empirical/Declared |
| `std.spores::b32_zero` | fn | `lib/std/spore.myc:145` | `fn b32_zero() => Binary{32}` | — | Empirical/Declared |
| `std.spores::b32_one` | fn | `lib/std/spore.myc:148` | `fn b32_one() => Binary{32}` | — | Empirical/Declared |
| `std.spores::b32_two` | fn | `lib/std/spore.myc:151` | `fn b32_two() => Binary{32}` | — | Empirical/Declared |
| `std.spores::b32_three` | fn | `lib/std/spore.myc:154` | `fn b32_three() => Binary{32}` | — | Empirical/Declared |
| `std.spores::b32_four` | fn | `lib/std/spore.myc:157` | `fn b32_four() => Binary{32}` | — | Empirical/Declared |
| `std.spores::nonempty` | fn | `lib/std/spore.myc:162` | `fn nonempty(b: Bytes) => Bool` | nonempty: True iff `b` has at least one byte (bytes_len is the Exact prim at Binary{32}; std.diag/std.recover precedent). | Empirical/Declared |
| `std.spores::bytes_eq_from` | fn | `lib/std/spore.myc:169` | `fn bytes_eq_from(a: Bytes, b: Bytes, i: Binary{32}, n: Binary{32}) => Bool` | bytes_eq_from: equal-length tail comparison over [i, n) — 4 bytes per recursion frame so a canonical 71-byte content-hash string fits the L1 depth budget (see the substitution note). Every bytes_get is bounds-guarded by the preceding `eq(index, n)` check, so the prim's out-of-range refusal is unreachable by construction (total on its domain). Exact. | Empirical/Declared |
| `std.spores::bytes_eq` | fn | `lib/std/spore.myc:199` | `fn bytes_eq(a: Bytes, b: Bytes) => Bool` | bytes_eq: whole byte-string equality composed from the D4 prims (no equality prim exists — the gap std.diag FLAGged; this is the honest composition, not a fabricated prim). Exact. | Empirical/Declared |
| `std.spores::vec_is_empty` | fn | `lib/std/spore.myc:206` | `fn vec_is_empty[A](xs: Vec[A]) => Bool` | vec_is_empty: structural emptiness — Exact. | Empirical/Declared |
| `std.spores::strength_rank` | fn | `lib/std/spore.myc:211` | `fn strength_rank(g: Guarantee) => Binary{8}` | — | Empirical/Declared |
| `std.spores::guarantee_eq` | fn | `lib/std/spore.myc:220` | `fn guarantee_eq(a: Guarantee, b: Guarantee) => Bool` | guarantee_eq: lattice-tag equality via the injective rank map — Exact. | Empirical/Declared |
| `std.spores::SporeErr` | type | `lib/std/spore.myc:227` | `type SporeErr = HashMismatch(Bytes, Bytes) \| PublishErr(Bytes) \| IoErr(Bytes)` | — | Empirical/Declared |
| `std.spores::SporeErr::HashMismatch` | ctor | `lib/std/spore.myc:227` | `HashMismatch(Bytes, Bytes)` | — | Empirical/Declared |
| `std.spores::SporeErr::IoErr` | ctor | `lib/std/spore.myc:227` | `IoErr(Bytes)` | — | Empirical/Declared |
| `std.spores::SporeErr::PublishErr` | ctor | `lib/std/spore.myc:227` | `PublishErr(Bytes)` | — | Empirical/Declared |
| `std.spores::spore_err_display` | fn | `lib/std/spore.myc:231` | `fn spore_err_display(e: SporeErr) => Bytes` | spore_err_display: the oracle's `Display for SporeErr`, byte-for-byte (differentially checked against `mycelium-std-spore` in std_spore.rs). Exact (deterministic concat of Exact prims). | Empirical/Declared |
| `std.spores::SporeUnit` | type | `lib/std/spore.myc:247` | `type SporeUnit = Spore(Bytes, Vec[Bytes], Option[ReconManifest])` | — | Empirical/Declared |
| `std.spores::SporeUnit::Spore` | ctor | `lib/std/spore.myc:247` | `Spore(Bytes, Vec[Bytes], Option[ReconManifest])` | — | Empirical/Declared |
| `std.spores::spore_carry` | fn | `lib/std/spore.myc:252` | `fn spore_carry(id: Bytes, surface: Vec[Bytes], m: Option[ReconManifest]) => SporeUnit` | spore_carry: wrap a kernel-minted identity + surface + manifest as a SporeUnit. This is the carry seam for FLAG-spore-2/-3 (the Rust constructors from_manifest/from_value run the M-368 pipeline; this handle only ever CARRIES their output). Exact; total. | Empirical/Declared |
| `std.spores::identity` | fn | `lib/std/spore.myc:256` | `fn identity(s: SporeUnit) => Bytes` | identity: the spore's canonical content-addressed identity (ADR-003). Exact; total. | Empirical/Declared |
| `std.spores::spore_surface` | fn | `lib/std/spore.myc:260` | `fn spore_surface(s: SporeUnit) => Vec[Bytes]` | spore_surface: the germination surface (export list). Exact; total. | Empirical/Declared |
| `std.spores::manifest_of` | fn | `lib/std/spore.myc:265` | `fn manifest_of(s: SporeUnit) => Option[ReconManifest]` | manifest_of: the reconstruction manifest, if carried — None is honest absence, never a fabricated empty manifest (C1/G2). Exact. | Empirical/Declared |
| `std.spores::verify_identity` | fn | `lib/std/spore.myc:272` | `fn verify_identity(declared: Bytes, recomputed: Bytes) => Result[Unit, SporeErr]` | verify_identity: the COMPARISON half of verify (FLAG-spore-2 carries the recomputation half to the kernel): equal → Ok(U); divergent → the explicit, named HashMismatch(declared, recomputed) — never a silent accept (C1/G2). `expected` is the declared identity, `found` the recomputed one (the oracle's field convention). Exact (deterministic comparison). | Empirical/Declared |
| `std.spores::verify` | fn | `lib/std/spore.myc:279` | `fn verify(s: SporeUnit, recomputed: Bytes) => Result[Unit, SporeErr]` | verify: the spore self-check against a kernel-recomputed identity. Exact. | Empirical/Declared |
| `std.spores::ReconMode` | type | `lib/std/spore.myc:284` | `type ReconMode = IndexedRetrieval \| CompositionalReconstruction` | — | Empirical/Declared |
| `std.spores::ReconMode::CompositionalReconstruction` | ctor | `lib/std/spore.myc:284` | `CompositionalReconstruction` | — | Empirical/Declared |
| `std.spores::ReconMode::IndexedRetrieval` | ctor | `lib/std/spore.myc:284` | `IndexedRetrieval` | — | Empirical/Declared |
| `std.spores::DecodeProcedure` | type | `lib/std/spore.myc:287` | `type DecodeProcedure = Cleanup \| Resonator` | The kernel DecodeProcedure — Cleanup (nearest-atom) or Resonator (probabilistic-only, FR-C2). | Empirical/Declared |
| `std.spores::DecodeProcedure::Cleanup` | ctor | `lib/std/spore.myc:287` | `Cleanup` | — | Empirical/Declared |
| `std.spores::DecodeProcedure::Resonator` | ctor | `lib/std/spore.myc:287` | `Resonator` | — | Empirical/Declared |
| `std.spores::Basis` | type | `lib/std/spore.myc:292` | `type Basis = ProvenThm(Bytes) \| EmpiricalFit(Binary{32}, Bytes) \| UserDeclared` | Basis — the kernel BoundBasis: the evidence class behind a bound certificate. ProvenThm carries its citation; EmpiricalFit its trials (Binary{32} domain substitution) + method; UserDeclared is the unvalidated assertion (always flagged). | Empirical/Declared |
| `std.spores::Basis::EmpiricalFit` | ctor | `lib/std/spore.myc:292` | `EmpiricalFit(Binary{32}, Bytes)` | — | Empirical/Declared |
| `std.spores::Basis::ProvenThm` | ctor | `lib/std/spore.myc:292` | `ProvenThm(Bytes)` | — | Empirical/Declared |
| `std.spores::Basis::UserDeclared` | ctor | `lib/std/spore.myc:292` | `UserDeclared` | — | Empirical/Declared |
| `std.spores::basis_strength` | fn | `lib/std/spore.myc:296` | `fn basis_strength(b: Basis) => Guarantee` | basis_strength: the honest strength a basis implies (the basis IS the evidence class — kernel BoundBasis::strength, mirrored exactly; VR-5). Exact. | Empirical/Declared |
| `std.spores::exceeds_empirical_ceiling` | fn | `lib/std/spore.myc:301` | `fn exceeds_empirical_ceiling(b: Basis) => Bool` | exceeds_empirical_ceiling: True iff the basis implies a strength STRONGER than Empirical (rank < 2 — i.e. Exact or Proven): the FR-C2 probabilistic-regrowth ceiling test. Exact. | Empirical/Declared |
| `std.spores::resonator_over_strength` | fn | `lib/std/spore.myc:306` | `fn resonator_over_strength(dec: DecodeProcedure, b: Basis) => Bool` | resonator_over_strength: the FR-C2 violation predicate — a Resonator decode whose basis exceeds the Empirical ceiling. A Cleanup decode has no resonator ceiling. Exact. | Empirical/Declared |
| `std.spores::MalformedManifest` | type | `lib/std/spore.myc:310` | `type MalformedManifest = ResonatorOverStrength \| KernelWf` | MalformedManifest — a refusal from manifest validation, explicitly named (C1/G2/G11). | Empirical/Declared |
| `std.spores::MalformedManifest::KernelWf` | ctor | `lib/std/spore.myc:310` | `KernelWf` | — | Empirical/Declared |
| `std.spores::MalformedManifest::ResonatorOverStrength` | ctor | `lib/std/spore.myc:310` | `ResonatorOverStrength` | — | Empirical/Declared |
| `std.spores::malformed_display` | fn | `lib/std/spore.myc:313` | `fn malformed_display(e: MalformedManifest) => Bytes` | malformed_display: the oracle's `Display for MalformedManifest`, byte-for-byte. Exact. | Empirical/Declared |
| `std.spores::ReconManifest` | type | `lib/std/spore.myc:325` | `type ReconManifest = Manifest(ReconMode, Bytes, Binary{32}, Vec[Bytes], DecodeProcedure, Basis)` | ReconManifest — the RFC-0003 §6 record: mode, model, dim, codebooks (kernel-minted content hashes, carried — FLAG-spore-2), decode procedure, and the bound's Basis (its {ε,δ} scalars are FLAG-spore-4). Construction goes through manifest_new (validates at build time) or manifest_validate (the deserialized-carry-in re-check). | Empirical/Declared |
| `std.spores::ReconManifest::Manifest` | ctor | `lib/std/spore.myc:325` | `Manifest(ReconMode, Bytes, Binary{32}, Vec[Bytes], DecodeProcedure, Basis)` | — | Empirical/Declared |
| `std.spores::manifest_new` | fn | `lib/std/spore.myc:332` | `fn manifest_new(mode: ReconMode, model: Bytes, dim: Binary{32}, codebooks: Vec[Bytes], dec: DecodeProcedure, basis: Basis) => Result[ReconManifest, MalformedManifest]` | manifest_new: build + validate a manifest. Ports the EXPRESSIBLE kernel checks (FLAG-spore-5): model nonempty, dim >= 1, codebooks nonempty, and the FR-C2 ceiling — every kernel refusal maps to Err(KernelWf), exactly as the oracle's new() maps every `WfError` to `MalformedManifest::KernelWf` (the ResonatorOverStrength variant is validate's, below). Exact (a pure predicate; same inputs, same outcome). | Empirical/Declared |
| `std.spores::manifest_validate` | fn | `lib/std/spore.myc:358` | `fn manifest_validate(m: ReconManifest) => Result[ReconManifest, MalformedManifest]` | manifest_validate: the carry-in path (deserialized/kernel-built manifests) — the explicit defense-in-depth FR-C2 re-check (C1/G2: never silently trust a carry-in). Unlike Rust (where the private inner field makes an over-strength ReconInfo unrepresentable), `.myc` constructors are open, so this check is genuinely reachable here. Exact. | Empirical/Declared |
| `std.spores::manifest_mode` | fn | `lib/std/spore.myc:368` | `fn manifest_mode(m: ReconManifest) => ReconMode` | — | Empirical/Declared |
| `std.spores::manifest_model` | fn | `lib/std/spore.myc:371` | `fn manifest_model(m: ReconManifest) => Bytes` | — | Empirical/Declared |
| `std.spores::manifest_dim` | fn | `lib/std/spore.myc:374` | `fn manifest_dim(m: ReconManifest) => Binary{32}` | — | Empirical/Declared |
| `std.spores::manifest_codebooks` | fn | `lib/std/spore.myc:377` | `fn manifest_codebooks(m: ReconManifest) => Vec[Bytes]` | — | Empirical/Declared |
| `std.spores::manifest_decode` | fn | `lib/std/spore.myc:380` | `fn manifest_decode(m: ReconManifest) => DecodeProcedure` | — | Empirical/Declared |
| `std.spores::declared_strength` | fn | `lib/std/spore.myc:386` | `fn declared_strength(m: ReconManifest) => Guarantee` | declared_strength: the strength from the manifest's bound certificate — derived from the basis, never fabricated (VR-5); always weaker-or-equal to Empirical for a Resonator decode (enforced at construction). Exact. | Empirical/Declared |
| `std.spores::RegrowthResult` | type | `lib/std/spore.myc:392` | `type RegrowthResult[T] = Regrown(T, Basis)` | — | Empirical/Declared |
| `std.spores::RegrowthResult::Regrown` | ctor | `lib/std/spore.myc:392` | `Regrown(T, Basis)` | — | Empirical/Declared |
| `std.spores::regrowth_new` | fn | `lib/std/spore.myc:398` | `fn regrowth_new[T](payload: T, basis: Basis) => Result[RegrowthResult[T], MalformedManifest]` | regrowth_new: construct a regrowth result, REFUSING a basis stronger than Empirical (the FR-C2/VR-5 probabilistic-regrowth ceiling) with the explicit ResonatorOverStrength — never a silent accept. This makes an over-strength regrowth unrepresentable through this constructor (the ceiling holds by construction, not by comment). Exact. | Empirical/Declared |
| `std.spores::regrowth_strength` | fn | `lib/std/spore.myc:405` | `fn regrowth_strength[T](r: RegrowthResult[T]) => Guarantee` | regrowth_strength: derived from the basis — never fabricated, never upgraded (VR-5). Exact. | Empirical/Declared |
| `std.spores::is_empirical` | fn | `lib/std/spore.myc:409` | `fn is_empirical[T](r: RegrowthResult[T]) => Bool` | is_empirical: True iff the strength is exactly Empirical (the expected resonator case). Exact. | Empirical/Declared |
| `std.spores::is_declared` | fn | `lib/std/spore.myc:413` | `fn is_declared[T](r: RegrowthResult[T]) => Bool` | is_declared: True iff the strength is Declared (the weakest; user-asserted only). Exact. | Empirical/Declared |
| `std.spores::DeployTarget` | type | `lib/std/spore.myc:419` | `type DeployTarget = InMemory \| Local(Bytes)` | — | Empirical/Declared |
| `std.spores::DeployTarget::InMemory` | ctor | `lib/std/spore.myc:419` | `InMemory` | — | Empirical/Declared |
| `std.spores::DeployTarget::Local` | ctor | `lib/std/spore.myc:419` | `Local(Bytes)` | — | Empirical/Declared |
| `std.spores::DeployVerification` | type | `lib/std/spore.myc:425` | `type DeployVerification = Verification(Bool, Bool)` | Verification(content_hash_canonical, no_opaque_lowering): the EXPLAIN-able record of what a successful deploy checked (VR-4/C4). Both True on every success — germinate refuses first otherwise. The no_opaque_lowering half is Declared strength (structural assertion, MLIR toolchain pending). | Empirical/Declared |
| `std.spores::DeployVerification::Verification` | ctor | `lib/std/spore.myc:425` | `Verification(Bool, Bool)` | — | Empirical/Declared |
| `std.spores::DeployError` | type | `lib/std/spore.myc:430` | `type DeployError = MissingInput \| AmbiguousInput(Vec[Bytes]) \| DeployHashMismatch(Bytes, Bytes) \| OpaqueStepDetected(Bytes)` | DeployError — an explicit deploy refusal, each variant naming its exact condition (G2/G11). `DeployHashMismatch(expected, actual)` renames the oracle's `HashMismatch` (constructor names are nodule-global; SporeErr owns `HashMismatch` — see the substitution note). | Empirical/Declared |
| `std.spores::DeployError::MissingInput` | ctor | `lib/std/spore.myc:431` | `MissingInput` | — | Empirical/Declared |
| `std.spores::DeployError::AmbiguousInput` | ctor | `lib/std/spore.myc:432` | `AmbiguousInput(Vec[Bytes])` | — | Empirical/Declared |
| `std.spores::DeployError::DeployHashMismatch` | ctor | `lib/std/spore.myc:433` | `DeployHashMismatch(Bytes, Bytes)` | — | Empirical/Declared |
| `std.spores::DeployError::OpaqueStepDetected` | ctor | `lib/std/spore.myc:434` | `OpaqueStepDetected(Bytes)` | — | Empirical/Declared |
| `std.spores::DeployResult` | type | `lib/std/spore.myc:438` | `type DeployResult = Deployed(Bytes, DeployVerification) \| Failed(DeployError)` | DeployResult — Deployed(spore_id, verification) on success; Failed(err) is the explicit, never-silent failure carrier (no partial/best-effort variant exists — C1/G2). | Empirical/Declared |
| `std.spores::DeployResult::Deployed` | ctor | `lib/std/spore.myc:438` | `Deployed(Bytes, DeployVerification)` | — | Empirical/Declared |
| `std.spores::DeployResult::Failed` | ctor | `lib/std/spore.myc:438` | `Failed(DeployError)` | — | Empirical/Declared |
| `std.spores::detect_opaque_step` | fn | `lib/std/spore.myc:444` | `fn detect_opaque_step(target: DeployTarget) => Option[Bytes]` | detect_opaque_step: the VR-4 hook — the v0 clean pipeline (pack → hash → deploy) contains no opaque steps by construction for the supported targets, so this returns None for both (the oracle's stub, mirrored exactly). Declared (structural assertion; MLIR toolchain pending — the oracle's own tag, carried). | Empirical/Declared |
| `std.spores::surface_is_ambiguous` | fn | `lib/std/spore.myc:449` | `fn surface_is_ambiguous(xs: Vec[Bytes]) => Bool` | surface_is_ambiguous: more than one exported surface symbol (the v0 ambiguity condition for an InMemory target with no entry-point selector). Exact. | Empirical/Declared |
| `std.spores::germinate_checked` | fn | `lib/std/spore.myc:456` | `fn germinate_checked(s: SporeUnit, recomputed: Bytes, target: DeployTarget) => Result[DeployResult, DeployError]` | germinate_checked: steps 3–5 of the oracle's germinate — hash canonicality (C4/ADR-003; mismatch → the named DeployHashMismatch, both hashes carried; a non-mismatch SporeErr from verify means the spore is not well-formed → MissingInput, the oracle's mapping), then the VR-4 opaque-step check, then success with the fully-checked Verification record. | Empirical/Declared |
| `std.spores::germinate` | fn | `lib/std/spore.myc:480` | `fn germinate(s: SporeUnit, recomputed: Bytes, target: DeployTarget) => Result[DeployResult, DeployError]` | germinate: the ADR-013 native germination entry point (the oracle's check order, mirrored): (1) missing input — a Local target with an empty path refuses before any other work (G2); (2) ambiguous input — an InMemory target over a multi-symbol surface refuses with ALL candidates listed (G11); (3) hash canonicality; (4) no-opaque-lowering; (5) success. The kernel-recomputed identity is an explicit input (FLAG-spore-2). Empirical — the native deploy path is not proven end-to-end (the oracle's tag; MLIR pending; VR-5: never upgraded even though this structural mirror is a pure function). | Empirical/Declared |
| `std.spores::deploy_error_display` | fn | `lib/std/spore.myc:500` | `fn deploy_error_display(e: DeployError) => Option[Bytes]` | deploy_error_display: the oracle's `Display for DeployError` — byte-for-byte for MissingInput / DeployHashMismatch / OpaqueStepDetected (differentially checked); None for AmbiguousInput (FLAG-spore-6: its rendering interpolates the decimal candidate count, which is not expressible — explicit absence, never a fabricated rendering). Exact where Some. | Empirical/Declared |
| `std.spores::explain_deployed` | fn | `lib/std/spore.myc:527` | `fn explain_deployed(spore_id: Bytes, v: DeployVerification) => Bytes` | explain_deployed: the Deployed-arm EXPLAIN of explain_deploy — a total, deterministic function of the deployed id + verification record, byte-for-byte the oracle's rendering (VR-4/SC-3/C3: it always mentions both the content-hash check and the opaque-lowering check — no silent omission). The Failed arm rides deploy_error_display (FLAG-spore-6). Exact. | Empirical/Declared |
| `std.spores::MatrixRow` | type | `lib/std/spore.myc:545` | `type MatrixRow = Row(Bytes, Bytes, Bytes, Bytes, Bool, Bytes)` | — | Empirical/Declared |
| `std.spores::MatrixRow::Row` | ctor | `lib/std/spore.myc:545` | `Row(Bytes, Bytes, Bytes, Bytes, Bool, Bytes)` | — | Empirical/Declared |
| `std.spores::row_op` | fn | `lib/std/spore.myc:548` | `fn row_op(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.spores::row_guarantee` | fn | `lib/std/spore.myc:551` | `fn row_guarantee(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.spores::row_fallibility` | fn | `lib/std/spore.myc:554` | `fn row_fallibility(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.spores::row_effects` | fn | `lib/std/spore.myc:557` | `fn row_effects(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.spores::row_explainable` | fn | `lib/std/spore.myc:560` | `fn row_explainable(r: MatrixRow) => Bool` | — | Empirical/Declared |
| `std.spores::row_never_silent` | fn | `lib/std/spore.myc:563` | `fn row_never_silent(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.spores::row_build` | fn | `lib/std/spore.myc:567` | `fn row_build() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_build_value` | fn | `lib/std/spore.myc:577` | `fn row_build_value() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_identity` | fn | `lib/std/spore.myc:587` | `fn row_identity() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_explain` | fn | `lib/std/spore.myc:597` | `fn row_explain() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_manifest_of` | fn | `lib/std/spore.myc:607` | `fn row_manifest_of() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_validate` | fn | `lib/std/spore.myc:617` | `fn row_validate() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_manifest_hash` | fn | `lib/std/spore.myc:627` | `fn row_manifest_hash() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_mode` | fn | `lib/std/spore.myc:630` | `fn row_mode() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_declared_strength` | fn | `lib/std/spore.myc:640` | `fn row_declared_strength() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_reconstruct` | fn | `lib/std/spore.myc:650` | `fn row_reconstruct() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_deploy` | fn | `lib/std/spore.myc:660` | `fn row_deploy() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_germinate` | fn | `lib/std/spore.myc:670` | `fn row_germinate() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_verify_hash_canonical` | fn | `lib/std/spore.myc:680` | `fn row_verify_hash_canonical() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_no_opaque` | fn | `lib/std/spore.myc:690` | `fn row_no_opaque() => MatrixRow` | — | Empirical/Declared |
| `std.spores::row_explain_deploy` | fn | `lib/std/spore.myc:700` | `fn row_explain_deploy() => MatrixRow` | — | Empirical/Declared |
| `std.spores::matrix` | fn | `lib/std/spore.myc:711` | `fn matrix() => Vec[MatrixRow]` | matrix: the full 15-row table, in the same order as guarantee_matrix.rs::MATRIX. | Empirical/Declared |
| `std.spores::matrix_len` | fn | `lib/std/spore.myc:731` | `fn matrix_len(xs: Vec[MatrixRow]) => Binary{8}` | matrix_len: the row count as Binary{8} (15 rows fits trivially). O(n) spine-walk. Exact. | Empirical/Declared |
| `std.spores::all_never_silent_nonempty` | fn | `lib/std/spore.myc:737` | `fn all_never_silent_nonempty(xs: Vec[MatrixRow]) => Bool` | — | Empirical/Declared |
| `std.spores::all_ops_nonempty` | fn | `lib/std/spore.myc:744` | `fn all_ops_nonempty(xs: Vec[MatrixRow]) => Bool` | every_row_has_nonempty_op_and_guarantee (basic completeness). | Empirical/Declared |
| `std.spores::all_guarantees_nonempty` | fn | `lib/std/spore.myc:747` | `fn all_guarantees_nonempty(xs: Vec[MatrixRow]) => Bool` | — | Empirical/Declared |
| `std.spores::manifest_rows_are_explainable` | fn | `lib/std/spore.myc:755` | `fn manifest_rows_are_explainable() => Bool` | manifest_rows_are_explainable: the 12 selecting/converting rows expose a C3 EXPLAIN artifact (guarantee_matrix.rs::manifest_rows_are_explain_able, read off the typed Bool column). | Empirical/Declared |
| `std.spores::accessor_rows_not_explainable` | fn | `lib/std/spore.myc:771` | `fn accessor_rows_not_explainable() => Bool` | accessor_rows_not_explainable: the pure-read rows (manifest_hash / mode / declared_strength) have no selection/approximation needing EXPLAIN (guarantee_matrix.rs precedent). | Empirical/Declared |

### std.swaps

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.swaps` | nodule | `lib/std/swap.myc:8` | `nodule std.swaps` | Self-hosted surface of `std.swap` — the never-silent representation-change library | Empirical/Declared |
| `std.swaps::Guarantee` | type | `lib/std/swap.myc:80` | `type Guarantee = GExact \| GProven \| GEmpirical \| GDeclared` | Honesty tags (VR-5 — carried at the SAME strength as the Rust source, never upgraded): - The matrix rows' `guarantee` fields mirror `GUARANTEE_MATRIX` exactly: `GExact` for the bijective class and the verdict/projection ops, `GProven` for f32->bf16 (ProvenThm basis, side-conditions checked kernel-side), `GEmpirical` for the Dense<->VSA class (probabilistic capacity bound). - The 7-row transcription itself is `Declared` (asserted data, structurally checked — the same strength as the Rust source's own `assert_matrix_invariants` tests). - Three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT) and the live Rust-oracle differential live in `crates/mycelium-l1/tests/std_swap.rs` and are `Empirical` (trials). - The swap instances below inherit the KERNEL's per-swap tag at runtime (`Exact` within range, derived by the engine, never asserted here): the L1 evaluator, the L0 interpreter, and the AOT path all dispatch to the same `mycelium-cert` engine the Rust oracle wraps (RFC-0031 D1), so this port validates the TRANSLATION SURFACE, not an independent second implementation — stated plainly (VR-5). ── Guarantee ──────────────────────────────────────────────────────────────────────────────────── The lattice tag Exact ⊐ Proven ⊐ Empirical ⊐ Declared (VR-5), as VALUE-level data. G-prefixed because the bare strength words are reserved (`T @ Exact` type-annotation surface) — FLAG-swap-0; same rename as std.diag. | Empirical/Declared |
| `std.swaps::Guarantee::GDeclared` | ctor | `lib/std/swap.myc:80` | `GDeclared` | — | Empirical/Declared |
| `std.swaps::Guarantee::GEmpirical` | ctor | `lib/std/swap.myc:80` | `GEmpirical` | — | Empirical/Declared |
| `std.swaps::Guarantee::GExact` | ctor | `lib/std/swap.myc:80` | `GExact` | — | Empirical/Declared |
| `std.swaps::Guarantee::GProven` | ctor | `lib/std/swap.myc:80` | `GProven` | — | Empirical/Declared |
| `std.swaps::CertKind` | type | `lib/std/swap.myc:87` | `type CertKind = KBijective \| KBounded \| KNone` | — | Empirical/Declared |
| `std.swaps::CertKind::KBijective` | ctor | `lib/std/swap.myc:87` | `KBijective` | — | Empirical/Declared |
| `std.swaps::CertKind::KBounded` | ctor | `lib/std/swap.myc:87` | `KBounded` | — | Empirical/Declared |
| `std.swaps::CertKind::KNone` | ctor | `lib/std/swap.myc:87` | `KNone` | — | Empirical/Declared |
| `std.swaps::Fallback` | type | `lib/std/swap.myc:97` | `type Fallback = UseReference` | — | Empirical/Declared |
| `std.swaps::Fallback::UseReference` | ctor | `lib/std/swap.myc:97` | `UseReference` | — | Empirical/Declared |
| `std.swaps::CheckError` | type | `lib/std/swap.myc:99` | `type CheckError = Refuted(Bytes) \| NotValidated(Bytes, Fallback)` | — | Empirical/Declared |
| `std.swaps::CheckError::NotValidated` | ctor | `lib/std/swap.myc:99` | `NotValidated(Bytes, Fallback)` | — | Empirical/Declared |
| `std.swaps::CheckError::Refuted` | ctor | `lib/std/swap.myc:99` | `Refuted(Bytes)` | — | Empirical/Declared |
| `std.swaps::refuted` | fn | `lib/std/swap.myc:102` | `fn refuted(detail: Bytes) => CheckError` | refuted: build the concrete-counterexample arm (the swap is WRONG). Total (Exact). | Empirical/Declared |
| `std.swaps::not_validated` | fn | `lib/std/swap.myc:107` | `fn not_validated(reason: Bytes) => CheckError` | not_validated: build the TV-incompleteness arm. The constructor PINS the explicit fallback (UseReference) — a NotValidated without a fallback path is unrepresentable through it. Total (Exact). | Empirical/Declared |
| `std.swaps::is_refuted` | fn | `lib/std/swap.myc:111` | `fn is_refuted(e: CheckError) => Bool` | is_refuted / is_not_validated: which arm — pure ADT dispatch. Total (Exact). | Empirical/Declared |
| `std.swaps::is_not_validated` | fn | `lib/std/swap.myc:114` | `fn is_not_validated(e: CheckError) => Bool` | — | Empirical/Declared |
| `std.swaps::has_explicit_fallback` | fn | `lib/std/swap.myc:119` | `fn has_explicit_fallback(e: CheckError) => Bool` | has_explicit_fallback: True iff the error carries a fallback route (the NotValidated arm — the "caller MUST take the enclosed Fallback" half of RFC-0002 §2). Total (Exact). | Empirical/Declared |
| `std.swaps::fallback_is_use_reference` | fn | `lib/std/swap.myc:124` | `fn fallback_is_use_reference(e: CheckError) => Bool` | fallback_is_use_reference: every representable fallback is UseReference (vacuously True on Refuted, which carries none) — the structural port of "always Fallback::UseReference". Total (Exact). | Empirical/Declared |
| `std.swaps::MatrixRow` | type | `lib/std/swap.myc:131` | `type MatrixRow = Row(Bytes, Guarantee, Bool, Bool, CertKind)` | — | Empirical/Declared |
| `std.swaps::MatrixRow::Row` | ctor | `lib/std/swap.myc:131` | `Row(Bytes, Guarantee, Bool, Bool, CertKind)` | — | Empirical/Declared |
| `std.swaps::row_op` | fn | `lib/std/swap.myc:134` | `fn row_op(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.swaps::row_guarantee` | fn | `lib/std/swap.myc:137` | `fn row_guarantee(r: MatrixRow) => Guarantee` | — | Empirical/Declared |
| `std.swaps::row_fallible` | fn | `lib/std/swap.myc:140` | `fn row_fallible(r: MatrixRow) => Bool` | — | Empirical/Declared |
| `std.swaps::row_cert_carrying` | fn | `lib/std/swap.myc:143` | `fn row_cert_carrying(r: MatrixRow) => Bool` | — | Empirical/Declared |
| `std.swaps::row_cert_kind` | fn | `lib/std/swap.myc:146` | `fn row_cert_kind(r: MatrixRow) => CertKind` | — | Empirical/Declared |
| `std.swaps::is_exact` | fn | `lib/std/swap.myc:150` | `fn is_exact(g: Guarantee) => Bool` | — | Empirical/Declared |
| `std.swaps::is_bijective` | fn | `lib/std/swap.myc:153` | `fn is_bijective(k: CertKind) => Bool` | — | Empirical/Declared |
| `std.swaps::is_bounded` | fn | `lib/std/swap.myc:156` | `fn is_bounded(k: CertKind) => Bool` | — | Empirical/Declared |
| `std.swaps::is_kind_none` | fn | `lib/std/swap.myc:159` | `fn is_kind_none(k: CertKind) => Bool` | — | Empirical/Declared |
| `std.swaps::bool_and` | fn | `lib/std/swap.myc:164` | `fn bool_and(a: Bool, b: Bool) => Bool` | bool_and: local total conjunction (no cross-leaf import; the std.diag/std.cmp self-containedness convention). Total over Bool x Bool (Exact). | Empirical/Declared |
| `std.swaps::bool_not` | fn | `lib/std/swap.myc:168` | `fn bool_not(a: Bool) => Bool` | bool_not: local total negation. Total (Exact). | Empirical/Declared |
| `std.swaps::nonempty` | fn | `lib/std/swap.myc:173` | `fn nonempty(b: Bytes) => Bool` | nonempty: True iff `b` has at least one byte. Delegates to the Exact `bytes_len` prim; the zero-comparison is the Exact `eq` prim at Binary{32} (the width `bytes_len` returns). Total (Exact). | Empirical/Declared |
| `std.swaps::row_bin_to_tern` | fn | `lib/std/swap.myc:180` | `fn row_bin_to_tern() => MatrixRow` | — | Empirical/Declared |
| `std.swaps::row_tern_to_bin` | fn | `lib/std/swap.myc:183` | `fn row_tern_to_bin() => MatrixRow` | — | Empirical/Declared |
| `std.swaps::row_f32_to_bf16` | fn | `lib/std/swap.myc:186` | `fn row_f32_to_bf16() => MatrixRow` | — | Empirical/Declared |
| `std.swaps::row_dense_to_vsa` | fn | `lib/std/swap.myc:189` | `fn row_dense_to_vsa() => MatrixRow` | — | Empirical/Declared |
| `std.swaps::row_vsa_to_dense` | fn | `lib/std/swap.myc:192` | `fn row_vsa_to_dense() => MatrixRow` | — | Empirical/Declared |
| `std.swaps::row_check_swap` | fn | `lib/std/swap.myc:195` | `fn row_check_swap() => MatrixRow` | — | Empirical/Declared |
| `std.swaps::row_explain` | fn | `lib/std/swap.myc:198` | `fn row_explain() => MatrixRow` | — | Empirical/Declared |
| `std.swaps::Vec` | type | `lib/std/swap.myc:202` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.swaps::Vec::Cons` | ctor | `lib/std/swap.myc:202` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.swaps::Vec::Nil` | ctor | `lib/std/swap.myc:202` | `Nil` | — | Empirical/Declared |
| `std.swaps::matrix` | fn | `lib/std/swap.myc:205` | `fn matrix() => Vec[MatrixRow]` | matrix: the full 7-row table, in the same order as lib.rs::GUARANTEE_MATRIX. | Empirical/Declared |
| `std.swaps::matrix_len` | fn | `lib/std/swap.myc:222` | `fn matrix_len(xs: Vec[MatrixRow]) => Binary{8}` | — | Empirical/Declared |
| `std.swaps::all_ops_nonempty` | fn | `lib/std/swap.myc:226` | `fn all_ops_nonempty(xs: Vec[MatrixRow]) => Bool` | all_ops_nonempty: no matrix row has an empty op name (lib.rs: "matrix row has empty op name"). | Empirical/Declared |
| `std.swaps::bijective_implies_exact` | fn | `lib/std/swap.myc:231` | `fn bijective_implies_exact(xs: Vec[MatrixRow]) => Bool` | bijective_implies_exact: a Bijective cert implies an Exact guarantee — the only genuinely bijective/exact swap class (RFC-0002 §4; lib.rs: "Bijective cert implies Exact guarantee"). | Empirical/Declared |
| `std.swaps::bounded_never_exact` | fn | `lib/std/swap.myc:243` | `fn bounded_never_exact(xs: Vec[MatrixRow]) => Bool` | bounded_never_exact: a Bounded cert is never Exact — Exact means no bound (M-I1; RFC-0001 §4.3; lib.rs: "Bounded cert cannot be Exact"). | Empirical/Declared |
| `std.swaps::nonfallible_no_cert_kind` | fn | `lib/std/swap.myc:259` | `fn nonfallible_no_cert_kind(xs: Vec[MatrixRow]) => Bool` | nonfallible_no_cert_kind: a non-fallible op emits no swap cert kind (lib.rs: "non-fallible op should not emit a swap cert_kind" — `explain` is cert_carrying but PROJECTS rather than emits, so its kind is KNone). | Empirical/Declared |
| `std.swaps::matrix_invariants_hold` | fn | `lib/std/swap.myc:272` | `fn matrix_invariants_hold() => Bool` | matrix_invariants_hold: the conjunction — the whole assert_matrix_invariants contract as one checkable value. (The Rust "cert_kind must be 'Bijective' or 'Bounded'" arm is discharged by the CertKind type — unrepresentable, see the CertKind doc.) | Empirical/Declared |
| `std.swaps::bin8_to_tern6` | fn | `lib/std/swap.myc:290` | `fn bin8_to_tern6(x: Binary{8}) => Ternary{6}` | Guarantee (VR-5, same strength as the Rust ops): Exact WITHIN RANGE — the kernel engine derives the tag per swap; out-of-range / illegal-pair inputs are explicit runtime REFUSALS (never a clamp, never a sentinel — C1), surfaced identically by the L1 evaluator, the L0 interpreter, and the AOT path (asserted in std_swap.rs). `rt` is the v0 policy name every execution path hashes through the same `policy-name.v0:` reference (NFR-7), the L1-corpus convention. | Empirical/Declared |
| `std.swaps::tern6_to_bin8` | fn | `lib/std/swap.myc:293` | `fn tern6_to_bin8(x: Ternary{6}) => Binary{8}` | — | Empirical/Declared |
| `std.swaps::bin4_to_tern3` | fn | `lib/std/swap.myc:296` | `fn bin4_to_tern3(x: Binary{4}) => Ternary{3}` | — | Empirical/Declared |
| `std.swaps::tern3_to_bin4` | fn | `lib/std/swap.myc:299` | `fn tern3_to_bin4(x: Ternary{3}) => Binary{4}` | — | Empirical/Declared |
| `std.swaps::roundtrip8` | fn | `lib/std/swap.myc:304` | `fn roundtrip8(x: Binary{8}) => Binary{8}` | roundtrip8 / roundtrip4: dec(enc(x)) — the LosslessWithinRange property (RFC-0002 §4) as a composable value; identity on the full corpus is asserted differentially in std_swap.rs. | Empirical/Declared |
| `std.swaps::roundtrip4` | fn | `lib/std/swap.myc:307` | `fn roundtrip4(x: Binary{4}) => Binary{4}` | — | Empirical/Declared |

### std.ternary

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.ternary` | nodule | `lib/std/ternary.myc:8` | `nodule std.ternary` | Self-hosted balanced-ternary value surface — Trit/Bit digit primitives, the exact | Empirical/Declared |
| `std.ternary::Option` | type | `lib/std/ternary.myc:91` | `type Option[A] = Some(A) \| None` | Open questions carried over from the crate (do not silently resolve): Q1 spelling (DN-02/06), Q2 lossy-scheme exclusion, Q3 caller-names-scheme vs RFC-0005 selector, Q4 width ceiling. ── Local Option / Result (redeclared; single-nodule harness — see the substitution note) ─────── | Empirical/Declared |
| `std.ternary::Option::None` | ctor | `lib/std/ternary.myc:91` | `None` | — | Empirical/Declared |
| `std.ternary::Option::Some` | ctor | `lib/std/ternary.myc:91` | `Some(A)` | — | Empirical/Declared |
| `std.ternary::Result` | type | `lib/std/ternary.myc:93` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `std.ternary::Result::Err` | ctor | `lib/std/ternary.myc:93` | `Err(E)` | — | Empirical/Declared |
| `std.ternary::Result::Ok` | ctor | `lib/std/ternary.myc:93` | `Ok(A)` | — | Empirical/Declared |
| `std.ternary::SInt` | type | `lib/std/ternary.myc:98` | `type SInt = SPos(Binary{16}) \| SNeg(Binary{16})` | — | Empirical/Declared |
| `std.ternary::SInt::SNeg` | ctor | `lib/std/ternary.myc:98` | `SNeg(Binary{16})` | — | Empirical/Declared |
| `std.ternary::SInt::SPos` | ctor | `lib/std/ternary.myc:98` | `SPos(Binary{16})` | — | Empirical/Declared |
| `std.ternary::Trit` | type | `lib/std/ternary.myc:101` | `type Trit = TNeg \| TZero \| TPos` | — | Empirical/Declared |
| `std.ternary::Trit::TNeg` | ctor | `lib/std/ternary.myc:101` | `TNeg` | — | Empirical/Declared |
| `std.ternary::Trit::TPos` | ctor | `lib/std/ternary.myc:101` | `TPos` | — | Empirical/Declared |
| `std.ternary::Trit::TZero` | ctor | `lib/std/ternary.myc:101` | `TZero` | — | Empirical/Declared |
| `std.ternary::trit_new` | fn | `lib/std/ternary.myc:106` | `fn trit_new(d: SInt) => Option[Trit]` | trit_new: construct a Trit from a signed integer. Guarantee: Exact. Returns None if d is outside {-1, 0, +1} — an explicit off-domain error, never a silent clamp (C1/G2). (SNeg(0) — non-canonical negative zero — reads as 0, mirroring Rust new(-0) == new(0).) | Empirical/Declared |
| `std.ternary::trit_digit` | fn | `lib/std/ternary.myc:120` | `fn trit_digit(t: Trit) => SInt` | trit_digit: the signed integer value of a trit (TNeg -> -1, TZero -> 0, TPos -> +1). Guarantee: Exact. Total — every trit has a unique integer value (C2). | Empirical/Declared |
| `std.ternary::trit_neg` | fn | `lib/std/ternary.myc:129` | `fn trit_neg(t: Trit) => Trit` | trit_neg: digit-wise negation — value(trit_neg(t)) = -value(t) exactly. Guarantee: Exact. Total — the balanced range is symmetric (no two's-complement asymmetry; binary-ternary.md §1). | Empirical/Declared |
| `std.ternary::trit_to_wire_byte` | fn | `lib/std/ternary.myc:134` | `fn trit_to_wire_byte(t: Trit) => Binary{8}` | trit_to_wire_byte: the MSB-first wire glyph as its ASCII byte — '-' = 45, '0' = 48, '+' = 43 (binary-ternary.md §1; FLAG-ternary-1 substitution for Rust's char). Guarantee: Exact. Total. | Empirical/Declared |
| `std.ternary::trit_from_wire_byte` | fn | `lib/std/ternary.myc:139` | `fn trit_from_wire_byte(b: Binary{8}) => Option[Trit]` | trit_from_wire_byte: parse an ASCII wire glyph back into a Trit. Guarantee: Exact. Returns None for any byte outside {45, 48, 43} (C1) — never a guessed digit. | Empirical/Declared |
| `std.ternary::Bit` | type | `lib/std/ternary.myc:149` | `type Bit = BZero \| BOne` | — | Empirical/Declared |
| `std.ternary::Bit::BOne` | ctor | `lib/std/ternary.myc:149` | `BOne` | — | Empirical/Declared |
| `std.ternary::Bit::BZero` | ctor | `lib/std/ternary.myc:149` | `BZero` | — | Empirical/Declared |
| `std.ternary::bit_new` | fn | `lib/std/ternary.myc:153` | `fn bit_new(d: SInt) => Option[Bit]` | bit_new: construct a Bit from a signed integer. Guarantee: Exact. Returns None if d is outside {0, 1} — explicit off-domain error (C1/G2); a negative input (SNeg with m >= 1) is None. | Empirical/Declared |
| `std.ternary::bit_digit` | fn | `lib/std/ternary.myc:163` | `fn bit_digit(b: Bit) => SInt` | bit_digit: the unsigned integer value of a bit (BZero -> 0, BOne -> 1). Guarantee: Exact. Total. | Empirical/Declared |
| `std.ternary::bit_and` | fn | `lib/std/ternary.myc:167` | `fn bit_and(a: Bit, b: Bit) => Bit` | bit_and / bit_or / bit_xor: total Boolean algebra. Guarantee: Exact (C2). | Empirical/Declared |
| `std.ternary::bit_or` | fn | `lib/std/ternary.myc:170` | `fn bit_or(a: Bit, b: Bit) => Bit` | — | Empirical/Declared |
| `std.ternary::bit_xor` | fn | `lib/std/ternary.myc:173` | `fn bit_xor(a: Bit, b: Bit) => Bit` | — | Empirical/Declared |
| `std.ternary::Trits` | type | `lib/std/ternary.myc:177` | `type Trits = TNil \| TCons(Trit, Trits)` | — | Empirical/Declared |
| `std.ternary::Trits::TCons` | ctor | `lib/std/ternary.myc:177` | `TCons(Trit, Trits)` | — | Empirical/Declared |
| `std.ternary::Trits::TNil` | ctor | `lib/std/ternary.myc:177` | `TNil` | — | Empirical/Declared |
| `std.ternary::trits_len` | fn | `lib/std/ternary.myc:182` | `fn trits_len(ts: Trits) => Binary{8}` | trits_len: the digit count, as Binary{8}. O(n) spine-walk (std.collections::len precedent); a string longer than 255 digits hits add_u's never-silent carry refusal — far past the m <= 10 codec ceiling (FLAG-ternary-2), so unreachable through this surface's own producers. | Empirical/Declared |
| `std.ternary::tsnoc` | fn | `lib/std/ternary.myc:186` | `fn tsnoc(ts: Trits, t: Trit) => Trits` | tsnoc: append one trit at the LSB end. O(n) spine-walk (std.collections::snoc precedent). | Empirical/Declared |
| `std.ternary::tconcat` | fn | `lib/std/ternary.myc:190` | `fn tconcat(a: Trits, b: Trits) => Trits` | tconcat: concatenate two trit strings. O(n) in the first spine. | Empirical/Declared |
| `std.ternary::zeros` | fn | `lib/std/ternary.myc:194` | `fn zeros(m: Binary{8}) => Trits` | zeros: the m-trit all-zero string (the balanced representation of 0 at width m). | Empirical/Declared |
| `std.ternary::ttake` | fn | `lib/std/ternary.myc:199` | `fn ttake(ts: Trits, n: Binary{8}) => Trits` | ttake / tdrop: the first n digits / everything after them. Total (short inputs yield what exists — used only on strings whose length the callers below have already established). | Empirical/Declared |
| `std.ternary::tdrop` | fn | `lib/std/ternary.myc:205` | `fn tdrop(ts: Trits, n: Binary{8}) => Trits` | — | Empirical/Declared |
| `std.ternary::all_zero` | fn | `lib/std/ternary.myc:212` | `fn all_zero(ts: Trits) => Bool` | all_zero: True iff every digit is TZero. Total (Exact — a finite-domain structural check). | Empirical/Declared |
| `std.ternary::b3x16` | fn | `lib/std/ternary.myc:221` | `fn b3x16(v: Binary{16}) => Binary{16}` | — | Empirical/Declared |
| `std.ternary::pow3` | fn | `lib/std/ternary.myc:226` | `fn pow3(m: Binary{8}) => Binary{16}` | pow3: 3^m at Binary{16}, for m <= 10 (3^10 = 59049 <= 65535; callers gate the ceiling — max_magnitude below refuses past it). Guarantee: Exact on the gated domain. | Empirical/Declared |
| `std.ternary::max_magnitude` | fn | `lib/std/ternary.myc:232` | `fn max_magnitude(m: Binary{8}) => Option[Binary{16}]` | max_magnitude: the maximum representable magnitude in m trits, (3^m - 1) / 2 — the symmetric range is [-max, +max]. Guarantee: Exact. Returns None for m > 10 (3^11 overflows Binary{16}) — the explicit width ceiling, mirroring Rust's m >= 41 => None (C1; FLAG-ternary-2 / spec Q4). | Empirical/Declared |
| `std.ternary::smul3` | fn | `lib/std/ternary.myc:239` | `fn smul3(x: SInt) => SInt` | smul3: signed times-3 (sign-magnitude — the sign is unchanged, the magnitude triples). | Empirical/Declared |
| `std.ternary::sadd_trit` | fn | `lib/std/ternary.myc:244` | `fn sadd_trit(x: SInt, t: Trit) => SInt` | sadd_trit: signed x + digit(t), canonical sign-magnitude (never SNeg(0)). Exact integer arithmetic; sub_u never underflows here because the magnitude cases are matched first. | Empirical/Declared |
| `std.ternary::t2i_go` | fn | `lib/std/ternary.myc:267` | `fn t2i_go(ts: Trits, acc: SInt) => SInt` | t2i_go: MSB-first Horner fold — acc' = acc\*3 + digit (binary-ternary.md §1). | Empirical/Declared |
| `std.ternary::trits_to_int` | fn | `lib/std/ternary.myc:275` | `fn trits_to_int(ts: Trits) => SInt` | trits_to_int: the integer denoted by an MSB-first trit string — value(t) = sum of digit(t_j) \* 3^(m-1-j) (Horner). The empty string denotes 0. Guarantee: Exact — an integer identity with no approximation (C2). Total for m <= 10; a longer string hits b3x16's never-silent add_u carry refusal (the Binary{16} analogue of Rust's i64 width ceiling, FLAG-ternary-2 — an explicit refusal, never a wrong value). | Empirical/Declared |
| `std.ternary::encode_mag` | fn | `lib/std/ternary.myc:284` | `fn encode_mag(u: Binary{16}, m: Binary{8}) => Option[Trits]` | encode_mag: the unique m-trit balanced representation of the NON-NEGATIVE magnitude u, MSB-first — the div_u/rem_u base conversion. Balanced digit choice per step (LSB-first): r = u rem 3 -> digit 0 (u' = u/3) \| +1 (u' = u/3) \| -1 (u' = (u+1)/3); the digit is appended at the LSB end (tsnoc), so the result is MSB-first. Returns None when the residual is nonzero after m digits — the value does not fit m trits (C1: explicit out-of-range, never truncation). (add_u(u, 1) cannot overflow: r = 2 implies u <= 65534.) | Empirical/Declared |
| `std.ternary::int_to_trits` | fn | `lib/std/ternary.myc:315` | `fn int_to_trits(v: SInt, m: Binary{8}) => Option[Trits]` | int_to_trits: the unique m-trit balanced representation of v, MSB-first. Guarantee: Exact. Returns None if v is outside [-(3^m-1)/2, +(3^m-1)/2] — an explicit out-of-range error, never a silent truncation or wrap (C1/G2). A negative v is the digit-wise negation of its magnitude's representation (balanced ternary is sign-symmetric — binary-ternary.md §1). Like the Rust crate, int_to_trits does not consult max_magnitude: 0 encodes at ANY width (all zeros). | Empirical/Declared |
| `std.ternary::trits_neg` | fn | `lib/std/ternary.myc:324` | `fn trits_neg(ts: Trits) => Trits` | — | Empirical/Declared |
| `std.ternary::TSum` | type | `lib/std/ternary.myc:328` | `type TSum = TS(Trit, Trit)` | TSum — a (digit, carry) pair from a balanced-ternary digit addition. | Empirical/Declared |
| `std.ternary::TSum::TS` | ctor | `lib/std/ternary.myc:328` | `TS(Trit, Trit)` | — | Empirical/Declared |
| `std.ternary::trit2_sum` | fn | `lib/std/ternary.myc:332` | `fn trit2_sum(x: Trit, y: Trit) => TSum` | trit2_sum: two-digit balanced sum — x + y = 3\*carry + digit with digit, carry in {-1, 0, +1} (the balanced-ternary half-adder table; Knuth 4.1). Guarantee: Exact. Total (9 cases). | Empirical/Declared |
| `std.ternary::trit_full_add` | fn | `lib/std/ternary.myc:343` | `fn trit_full_add(x: Trit, y: Trit, c: Trit) => TSum` | trit_full_add: three-digit balanced sum (x + y + carry-in). The two partial carries c1, c2 satisfy \|c1 + c2\| <= 1 (x+y+c is in [-3, 3], so 3\*(c1+c2) = total - digit is in [-4, 4]), so their own sum has a zero carry — the discarded `_` below is structurally always TZero (grounded here, not silently dropped — G2). Guarantee: Exact. Total. | Empirical/Declared |
| `std.ternary::TCarry` | type | `lib/std/ternary.myc:352` | `type TCarry = TC(Trits, Trit)` | TCarry — (partial sum, carry toward the MSB) from adding two equal-length tails. | Empirical/Declared |
| `std.ternary::TCarry::TC` | ctor | `lib/std/ternary.myc:352` | `TC(Trits, Trit)` | — | Empirical/Declared |
| `std.ternary::addc` | fn | `lib/std/ternary.myc:356` | `fn addc(a: Trits, b: Trits) => Option[TCarry]` | addc: ripple-carry addition of two MSB-first strings — recurses to the LSB end first, then adds each digit column with the carry coming back up. None on unequal lengths (C1). | Empirical/Declared |
| `std.ternary::trits_add` | fn | `lib/std/ternary.myc:378` | `fn trits_add(a: Trits, b: Trits) => Option[Trits]` | trits_add: fixed-width balanced-ternary addition a + b. Guarantee: Exact. Returns None on fixed-width overflow — a nonzero final carry, i.e. the true sum lies outside [-max_magnitude(m), +max_magnitude(m)] — and None if the widths differ. Never silently wraps (C1/G2). (The nonzero-carry test IS the overflow test: value(digits) + 3^m \* carry = a + b, and \|a + b\| <= 2\*max < 3^m forces carry = 0 exactly when the sum is representable.) | Empirical/Declared |
| `std.ternary::trits_sub` | fn | `lib/std/ternary.myc:389` | `fn trits_sub(a: Trits, b: Trits) => Option[Trits]` | trits_sub: fixed-width subtraction a - b = trits_add(a, trits_neg(b)) — the same identity the Rust crate uses. Guarantee: Exact. None on overflow or unequal widths (C1/G2). | Empirical/Declared |
| `std.ternary::t3x` | fn | `lib/std/ternary.myc:395` | `fn t3x(x: Trits) => Option[Trits]` | t3x: fixed-width times-3 as two checked additions (x + x + x). Option-propagating so any out-of-range intermediate stays never-silent; trits_mul only calls it at the doubled width, where the Horner intermediates provably fit (see trits_mul). | Empirical/Declared |
| `std.ternary::term_of` | fn | `lib/std/ternary.myc:399` | `fn term_of(d: Trit, aw: Trits) => Trits` | term_of: digit \* a — the partial product row (TPos -> a, TZero -> 0, TNeg -> -a). | Empirical/Declared |
| `std.ternary::mul_go` | fn | `lib/std/ternary.myc:403` | `fn mul_go(bs: Trits, aw: Trits, acc: Trits) => Option[Trits]` | mul_go: MSB-first Horner over b's digits at the doubled width — acc' = acc\*3 + digit\*a. | Empirical/Declared |
| `std.ternary::trits_mul` | fn | `lib/std/ternary.myc:421` | `fn trits_mul(a: Trits, b: Trits) => Option[Trits]` | trits_mul: fixed-width balanced-ternary multiplication a \* b. Computes the full 2m-trit product (both operands zero-padded to width 2m; every Horner intermediate is bounded by (3^(k+m)-1)/2 for k processed digits, so the 2m-width additions never overflow) and returns the low m trits iff the high m trits are all zero — otherwise None (overflow, explicit). Also None if the widths differ. Guarantee: Exact — never a silent truncation (C1/G2). | Empirical/Declared |
| `std.ternary::Scheme` | type | `lib/std/ternary.myc:437` | `type Scheme = I2S \| Tl1 \| Tl2` | — | Empirical/Declared |
| `std.ternary::Scheme::I2S` | ctor | `lib/std/ternary.myc:437` | `I2S` | — | Empirical/Declared |
| `std.ternary::Scheme::Tl1` | ctor | `lib/std/ternary.myc:437` | `Tl1` | — | Empirical/Declared |
| `std.ternary::Scheme::Tl2` | ctor | `lib/std/ternary.myc:437` | `Tl2` | — | Empirical/Declared |
| `std.ternary::scheme_trits_per_byte` | fn | `lib/std/ternary.myc:442` | `fn scheme_trits_per_byte(s: Scheme) => Binary{8}` | scheme_trits_per_byte / scheme_group_size: I2S packs 4 trits/byte (2 bits each); TL1/TL2 pack 5 trits/byte (base-3 in 243 <= 256). The group size equals the per-byte count; a pack call on a non-multiple is an explicit Err(Misaligned) (RFC-0004 §5 "align to SIMD width"; C1). | Empirical/Declared |
| `std.ternary::scheme_group_size` | fn | `lib/std/ternary.myc:445` | `fn scheme_group_size(s: Scheme) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::PackError` | type | `lib/std/ternary.myc:450` | `type PackError = OffGrid \| Misaligned` | PackError: explicit pack/unpack errors (C1/G2 — never a sentinel). OffGrid: an encoding falls outside the scheme's alphabet. Misaligned: the trit count is not a multiple of the group size. | Empirical/Declared |
| `std.ternary::PackError::Misaligned` | ctor | `lib/std/ternary.myc:450` | `Misaligned` | — | Empirical/Declared |
| `std.ternary::PackError::OffGrid` | ctor | `lib/std/ternary.myc:450` | `OffGrid` | — | Empirical/Declared |
| `std.ternary::SelectionNote` | type | `lib/std/ternary.myc:454` | `type SelectionNote = ExplicitCaller` | SelectionNote: how the scheme was selected. v0: the caller named it explicitly (FLAG Q3 — the RFC-0005 selector would replace this with a PolicyRef; the structure is forward-compatible). | Empirical/Declared |
| `std.ternary::SelectionNote::ExplicitCaller` | ctor | `lib/std/ternary.myc:454` | `ExplicitCaller` | — | Empirical/Declared |
| `std.ternary::ByteList` | type | `lib/std/ternary.myc:458` | `type ByteList = BNil \| BCons(Binary{8}, ByteList)` | ByteList — the packed bytes as a cons-list of Binary{8} (FLAG-ternary-3 substitution for Vec<u8>: no surface prim constructs a kernel Bytes from computed bytes). | Empirical/Declared |
| `std.ternary::ByteList::BCons` | ctor | `lib/std/ternary.myc:458` | `BCons(Binary{8}, ByteList)` | — | Empirical/Declared |
| `std.ternary::ByteList::BNil` | ctor | `lib/std/ternary.myc:458` | `BNil` | — | Empirical/Declared |
| `std.ternary::bytelist_len` | fn | `lib/std/ternary.myc:460` | `fn bytelist_len(bs: ByteList) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::Packed` | type | `lib/std/ternary.myc:467` | `type Packed = MkPacked(ByteList, Scheme, Binary{8})` | Packed — bytes + the inspectable Meta.physical record: MkPacked(bytes, scheme, trit_count). Two packings of the same trits under different schemes are the same LOGICAL value (DN-01 — packing is not a type distinction; C4: metadata is not identity). Field privacy is not expressible (FLAG-ternary-4) — unpack defends with explicit Err(OffGrid) instead. | Empirical/Declared |
| `std.ternary::Packed::MkPacked` | ctor | `lib/std/ternary.myc:467` | `MkPacked(ByteList, Scheme, Binary{8})` | — | Empirical/Declared |
| `std.ternary::ExplainRecord` | type | `lib/std/ternary.myc:471` | `type ExplainRecord = ExplainRec(Scheme, SelectionNote, Binary{8}, Binary{8})` | ExplainRecord — the inspectable EXPLAIN record (C3/NFR-1/SC-3): ExplainRec(scheme, selection, trit_count, byte_count). | Empirical/Declared |
| `std.ternary::ExplainRecord::ExplainRec` | ctor | `lib/std/ternary.myc:471` | `ExplainRec(Scheme, SelectionNote, Binary{8}, Binary{8})` | — | Empirical/Declared |
| `std.ternary::i2s_encode` | fn | `lib/std/ternary.myc:476` | `fn i2s_encode(t: Trit) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::i2s_byte` | fn | `lib/std/ternary.myc:479` | `fn i2s_byte(t0: Trit, t1: Trit, t2: Trit, t3: Trit) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::i2s_decode` | fn | `lib/std/ternary.myc:489` | `fn i2s_decode(bits: Binary{8}) => Result[Trit, PackError]` | i2s_decode: the low 2 bits of `bits` back to a Trit; 0b10 is an explicit Err(OffGrid) (C1). | Empirical/Declared |
| `std.ternary::pack_i2s` | fn | `lib/std/ternary.myc:501` | `fn pack_i2s(ts: Trits) => Option[ByteList]` | pack_i2s: 4 trits per byte; a ragged tail (1-3 leftover trits) is None — unreachable after pack's alignment check, kept explicit rather than silently padded (G2). | Empirical/Declared |
| `std.ternary::unpack_i2s` | fn | `lib/std/ternary.myc:519` | `fn unpack_i2s(bs: ByteList) => Result[Trits, PackError]` | — | Empirical/Declared |
| `std.ternary::b3x8` | fn | `lib/std/ternary.myc:543` | `fn b3x8(v: Binary{8}) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::tl1_encode` | fn | `lib/std/ternary.myc:546` | `fn tl1_encode(t: Trit) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::tl1_decode_digit` | fn | `lib/std/ternary.myc:551` | `fn tl1_decode_digit(d: Binary{8}) => Trit` | tl1_decode_digit: base-3 digit back to a Trit. Total on {0, 1, 2} — every caller feeds it rem_u(_, 3), whose result is always in {0, 1, 2} (the >242-byte case is caught separately). | Empirical/Declared |
| `std.ternary::tl1_byte` | fn | `lib/std/ternary.myc:557` | `fn tl1_byte(t0: Trit, t1: Trit, t2: Trit, t3: Trit, t4: Trit) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::pack_tl1` | fn | `lib/std/ternary.myc:564` | `fn pack_tl1(ts: Trits) => Option[ByteList]` | pack_tl1: 5 trits per byte; a ragged tail is None (unreachable after the alignment check; G2). | Empirical/Declared |
| `std.ternary::tl1_group` | fn | `lib/std/ternary.myc:588` | `fn tl1_group(byte: Binary{8}) => Result[Trits, PackError]` | tl1_group: one byte back to its 5-trit group (MSB-first). The five base-3 digits come off LSB-first via rem_u/div_u; a residual above 0 after five divisions means the byte value was > 242 — an explicit Err(OffGrid), never a silently wrapped group (C1/G2). | Empirical/Declared |
| `std.ternary::unpack_tl1` | fn | `lib/std/ternary.myc:603` | `fn unpack_tl1(bs: ByteList) => Result[Trits, PackError]` | — | Empirical/Declared |
| `std.ternary::pack_tl2` | fn | `lib/std/ternary.myc:617` | `fn pack_tl2(ts: Trits) => Option[ByteList]` | — | Empirical/Declared |
| `std.ternary::tl2_complement` | fn | `lib/std/ternary.myc:620` | `fn tl2_complement(bs: ByteList) => ByteList` | — | Empirical/Declared |
| `std.ternary::unpack_tl2` | fn | `lib/std/ternary.myc:623` | `fn unpack_tl2(bs: ByteList) => Result[Trits, PackError]` | — | Empirical/Declared |
| `std.ternary::pack` | fn | `lib/std/ternary.myc:645` | `fn pack(ts: Trits, s: Scheme) => Result[Packed, PackError]` | — | Empirical/Declared |
| `std.ternary::unpack` | fn | `lib/std/ternary.myc:660` | `fn unpack(p: Packed) => Result[Trits, PackError]` | unpack: the packed trits back out — lossless on every pack-produced value (Exact; DN-01 §2). FLAG-ternary-4: without field privacy a forged Packed is constructible, so unpack returns Result — a corrupt byte (I2S 0b10 code, TL1/TL2 byte > 242) or a trit_count that disagrees with the decoded length is an explicit Err(OffGrid), never a panic or a silent wrong value. | Empirical/Declared |
| `std.ternary::unpack_check` | fn | `lib/std/ternary.myc:670` | `fn unpack_check(r: Result[Trits, PackError], n: Binary{8}) => Result[Trits, PackError]` | unpack_check: the defensive trit-count agreement gate (FLAG-ternary-4). | Empirical/Declared |
| `std.ternary::scheme_of` | fn | `lib/std/ternary.myc:678` | `fn scheme_of(p: Packed) => Scheme` | scheme_of: the scheme used to pack — the inspectable Meta.physical record (C3/NFR-1). Guarantee: Exact. Total. | Empirical/Declared |
| `std.ternary::packed_bytes` | fn | `lib/std/ternary.myc:682` | `fn packed_bytes(p: Packed) => ByteList` | packed_bytes / packed_trit_count: read-only projections of the packed value. | Empirical/Declared |
| `std.ternary::packed_trit_count` | fn | `lib/std/ternary.myc:685` | `fn packed_trit_count(p: Packed) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::explain` | fn | `lib/std/ternary.myc:690` | `fn explain(p: Packed) => ExplainRecord` | explain: the full EXPLAIN record — scheme, how it was selected (v0: explicit caller — FLAG Q3), trit count, and byte count (C3/G11/NFR-1/SC-3). Guarantee: Exact. Total. | Empirical/Declared |
| `std.ternary::Tag` | type | `lib/std/ternary.myc:698` | `type Tag = GExact \| GProven \| GEmpirical \| GDeclared` | — | Empirical/Declared |
| `std.ternary::Tag::GDeclared` | ctor | `lib/std/ternary.myc:698` | `GDeclared` | — | Empirical/Declared |
| `std.ternary::Tag::GEmpirical` | ctor | `lib/std/ternary.myc:698` | `GEmpirical` | — | Empirical/Declared |
| `std.ternary::Tag::GExact` | ctor | `lib/std/ternary.myc:698` | `GExact` | — | Empirical/Declared |
| `std.ternary::Tag::GProven` | ctor | `lib/std/ternary.myc:698` | `GProven` | — | Empirical/Declared |
| `std.ternary::Fallibility` | type | `lib/std/ternary.myc:702` | `type Fallibility = FTotal \| FNoneOnOffDomain(Bytes) \| FErrOn(Bytes)` | Fallibility: total, explicit-None off-domain (with the named condition), or explicit-Err (with the named error set). Prose conditions are Bytes (textual string literals). | Empirical/Declared |
| `std.ternary::Fallibility::FErrOn` | ctor | `lib/std/ternary.myc:702` | `FErrOn(Bytes)` | — | Empirical/Declared |
| `std.ternary::Fallibility::FNoneOnOffDomain` | ctor | `lib/std/ternary.myc:702` | `FNoneOnOffDomain(Bytes)` | — | Empirical/Declared |
| `std.ternary::Fallibility::FTotal` | ctor | `lib/std/ternary.myc:702` | `FTotal` | — | Empirical/Declared |
| `std.ternary::Explainable` | type | `lib/std/ternary.myc:705` | `type Explainable = ExplainYes \| ExplainNA` | Explainable: whether the op exposes an inspectable EXPLAIN artifact (C3). Only the pack ops do. | Empirical/Declared |
| `std.ternary::Explainable::ExplainNA` | ctor | `lib/std/ternary.myc:705` | `ExplainNA` | — | Empirical/Declared |
| `std.ternary::Explainable::ExplainYes` | ctor | `lib/std/ternary.myc:705` | `ExplainYes` | — | Empirical/Declared |
| `std.ternary::OpGuarantee` | type | `lib/std/ternary.myc:709` | `type OpGuarantee = OpRow(Bytes, Tag, Fallibility, Bytes, Explainable)` | OpGuarantee — one matrix row: OpRow(op, tag, fallibility, effects, explainable). The op name and effects are prose Bytes; tag/fallibility/explainable are typed ADTs (never stringly). | Empirical/Declared |
| `std.ternary::OpGuarantee::OpRow` | ctor | `lib/std/ternary.myc:709` | `OpRow(Bytes, Tag, Fallibility, Bytes, Explainable)` | — | Empirical/Declared |
| `std.ternary::Rows` | type | `lib/std/ternary.myc:711` | `type Rows = RNil \| RCons(OpGuarantee, Rows)` | — | Empirical/Declared |
| `std.ternary::Rows::RCons` | ctor | `lib/std/ternary.myc:711` | `RCons(OpGuarantee, Rows)` | — | Empirical/Declared |
| `std.ternary::Rows::RNil` | ctor | `lib/std/ternary.myc:711` | `RNil` | — | Empirical/Declared |
| `std.ternary::row_trit_new` | fn | `lib/std/ternary.myc:713` | `fn row_trit_new() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_bit_new` | fn | `lib/std/ternary.myc:716` | `fn row_bit_new() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_trit_digit` | fn | `lib/std/ternary.myc:719` | `fn row_trit_digit() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_trit_neg` | fn | `lib/std/ternary.myc:722` | `fn row_trit_neg() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_bit_digit` | fn | `lib/std/ternary.myc:725` | `fn row_bit_digit() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_bit_and` | fn | `lib/std/ternary.myc:728` | `fn row_bit_and() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_bit_or` | fn | `lib/std/ternary.myc:731` | `fn row_bit_or() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_bit_xor` | fn | `lib/std/ternary.myc:734` | `fn row_bit_xor() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_trits_to_int` | fn | `lib/std/ternary.myc:739` | `fn row_trits_to_int() => OpGuarantee` | trits_to_int: total on the m <= 10 domain; past it the b3x16 tripling refuses never-silently (FLAG-ternary-2 — the ported analogue of the Rust row's i64-total contract). | Empirical/Declared |
| `std.ternary::row_int_to_trits` | fn | `lib/std/ternary.myc:742` | `fn row_int_to_trits() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_trits_neg` | fn | `lib/std/ternary.myc:751` | `fn row_trits_neg() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_trits_add` | fn | `lib/std/ternary.myc:754` | `fn row_trits_add() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_trits_sub` | fn | `lib/std/ternary.myc:763` | `fn row_trits_sub() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_trits_mul` | fn | `lib/std/ternary.myc:772` | `fn row_trits_mul() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_pack` | fn | `lib/std/ternary.myc:781` | `fn row_pack() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_unpack` | fn | `lib/std/ternary.myc:786` | `fn row_unpack() => OpGuarantee` | unpack: Total in the Rust crate (privacy-guarded); ported as explicit Err(OffGrid) on a forged/corrupt Packed (FLAG-ternary-4 — a stated fallibility widening, not a tag change). | Empirical/Declared |
| `std.ternary::row_scheme_of` | fn | `lib/std/ternary.myc:795` | `fn row_scheme_of() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::row_explain` | fn | `lib/std/ternary.myc:798` | `fn row_explain() => OpGuarantee` | — | Empirical/Declared |
| `std.ternary::matrix` | fn | `lib/std/ternary.myc:802` | `fn matrix() => Rows` | matrix: all 18 rows, in the same order as guarantee_matrix.rs::MATRIX. | Empirical/Declared |
| `std.ternary::bool_and` | fn | `lib/std/ternary.myc:826` | `fn bool_and(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `std.ternary::nonempty` | fn | `lib/std/ternary.myc:830` | `fn nonempty(b: Bytes) => Bool` | nonempty: True iff the Bytes has at least one byte (bytes_len is Exact; eq at Binary{32}). | Empirical/Declared |
| `std.ternary::is_exact_tag` | fn | `lib/std/ternary.myc:833` | `fn is_exact_tag(t: Tag) => Bool` | — | Empirical/Declared |
| `std.ternary::row_tag` | fn | `lib/std/ternary.myc:836` | `fn row_tag(r: OpGuarantee) => Tag` | — | Empirical/Declared |
| `std.ternary::row_fallibility` | fn | `lib/std/ternary.myc:839` | `fn row_fallibility(r: OpGuarantee) => Fallibility` | — | Empirical/Declared |
| `std.ternary::row_effects` | fn | `lib/std/ternary.myc:842` | `fn row_effects(r: OpGuarantee) => Bytes` | — | Empirical/Declared |
| `std.ternary::row_explainable` | fn | `lib/std/ternary.myc:845` | `fn row_explainable(r: OpGuarantee) => Explainable` | — | Empirical/Declared |
| `std.ternary::matrix_len` | fn | `lib/std/ternary.myc:848` | `fn matrix_len(xs: Rows) => Binary{8}` | — | Empirical/Declared |
| `std.ternary::all_exact` | fn | `lib/std/ternary.myc:853` | `fn all_exact(xs: Rows) => Bool` | all_exact: every row must tag GExact (C2/VR-5 — a non-Exact row would be a dishonest downgrade of an exact fact; a mutant flipping any row fails this). | Empirical/Declared |
| `std.ternary::all_effects_nonempty` | fn | `lib/std/ternary.myc:858` | `fn all_effects_nonempty(xs: Rows) => Bool` | all_effects_nonempty: every row states its effects — all "none" here (C6/RFC-0014). The "== none" content assertion needs a bytes_eq prim (FLAG-ternary-5); non-emptiness is portable. | Empirical/Declared |
| `std.ternary::fallible_rows_name_their_error_set` | fn | `lib/std/ternary.myc:866` | `fn fallible_rows_name_their_error_set(xs: Rows) => Bool` | fallible_rows_name_their_error_set: every NoneOnOffDomain/ErrOn row carries a non-empty condition/error-set string (C1/G2 — the error set must be named). Total rows are vacuous. | Empirical/Declared |
| `std.ternary::count_explainable` | fn | `lib/std/ternary.myc:883` | `fn count_explainable(xs: Rows) => Binary{8}` | count_explainable: how many rows expose an EXPLAIN artifact — exactly the 4 pack ops (pack/unpack/scheme_of/explain). The name-keyed form of this check needs bytes_eq (FLAG-ternary-5); the count form is the portable structural equivalent. | Empirical/Declared |

### std.testing

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.testing` | nodule | `lib/std/testing.myc:8` | `nodule std.testing` | Self-hosted structural testing surface — property / golden / differential harness | Empirical/Declared |
| `std.testing::Option` | type | `lib/std/testing.myc:126` | `type Option[Opt] = Some(Opt) \| None` | Never-silent floor (C1/G2) carried over intact: Skipped and Undetermined are first-class variants aggregated DISTINCTLY from Pass ("green" = checked-and-passed, never did-not-check); a missing/mismatched golden baseline is Skipped(NeedsRecord), never an auto-accept; an unavailable differential backend is Skipped(BackendUnavailable), never a silent pass; a generator that produces nothing is Skipped(NeedsRecord). Guarantee tags (VR-5, carried at the SAME strength as the Rust crate's matrix): every harness op is an Exact MECHANISM (a verdict is an exact, deterministic function of the run) and the harness never inflates the subject's tag — a passing for_all backs an Empirical claim about the property, never Proven; there is no op here that upgrades "passed N trials" into a proof. Three-way differential agreement is Empirical (trials, std_testing.rs). ── local mirrors (single-nodule harness — see the substitution notes above) ──────────────────── | Empirical/Declared |
| `std.testing::Option::None` | ctor | `lib/std/testing.myc:126` | `None` | — | Empirical/Declared |
| `std.testing::Option::Some` | ctor | `lib/std/testing.myc:126` | `Some(Opt)` | — | Empirical/Declared |
| `std.testing::Vec` | type | `lib/std/testing.myc:128` | `type Vec[A] = Nil \| Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.testing::Vec::Cons` | ctor | `lib/std/testing.myc:128` | `Cons(A, Vec[A])` | — | Empirical/Declared |
| `std.testing::Vec::Nil` | ctor | `lib/std/testing.myc:128` | `Nil` | — | Empirical/Declared |
| `std.testing::Unit` | type | `lib/std/testing.myc:132` | `type Unit = U` | Unit — the nullary-marker substitute for the missing `() => T` closure-domain form (std.error.myc precedent). | Empirical/Declared |
| `std.testing::Unit::U` | ctor | `lib/std/testing.myc:132` | `U` | — | Empirical/Declared |
| `std.testing::Guarantee` | type | `lib/std/testing.myc:136` | `type Guarantee = GExact \| GProven \| GEmpirical \| GDeclared` | Guarantee — the kernel GuaranteeStrength lattice as value-level data, G-prefixed (reserved-keyword substitution; std.diag/std.recover precedent). | Empirical/Declared |
| `std.testing::Guarantee::GDeclared` | ctor | `lib/std/testing.myc:136` | `GDeclared` | — | Empirical/Declared |
| `std.testing::Guarantee::GEmpirical` | ctor | `lib/std/testing.myc:136` | `GEmpirical` | — | Empirical/Declared |
| `std.testing::Guarantee::GExact` | ctor | `lib/std/testing.myc:136` | `GExact` | — | Empirical/Declared |
| `std.testing::Guarantee::GProven` | ctor | `lib/std/testing.myc:136` | `GProven` | — | Empirical/Declared |
| `std.testing::SkipReason` | type | `lib/std/testing.myc:140` | `type SkipReason = Ignored \| UnmetPrecondition \| NeedsRecord \| BackendUnavailable \| ToolMissing` | — | Empirical/Declared |
| `std.testing::SkipReason::BackendUnavailable` | ctor | `lib/std/testing.myc:140` | `BackendUnavailable` | — | Empirical/Declared |
| `std.testing::SkipReason::Ignored` | ctor | `lib/std/testing.myc:140` | `Ignored` | — | Empirical/Declared |
| `std.testing::SkipReason::NeedsRecord` | ctor | `lib/std/testing.myc:140` | `NeedsRecord` | — | Empirical/Declared |
| `std.testing::SkipReason::ToolMissing` | ctor | `lib/std/testing.myc:140` | `ToolMissing` | — | Empirical/Declared |
| `std.testing::SkipReason::UnmetPrecondition` | ctor | `lib/std/testing.myc:140` | `UnmetPrecondition` | — | Empirical/Declared |
| `std.testing::UndetReason` | type | `lib/std/testing.myc:144` | `type UndetReason = OracleUnavailable \| BudgetExhaustedInconclusive \| NonDeterministicInput` | UndetReason — ran but could not decide; distinct from both Pass and Skipped. A non-decision is never a Pass (C1/G2). | Empirical/Declared |
| `std.testing::UndetReason::BudgetExhaustedInconclusive` | ctor | `lib/std/testing.myc:144` | `BudgetExhaustedInconclusive` | — | Empirical/Declared |
| `std.testing::UndetReason::NonDeterministicInput` | ctor | `lib/std/testing.myc:144` | `NonDeterministicInput` | — | Empirical/Declared |
| `std.testing::UndetReason::OracleUnavailable` | ctor | `lib/std/testing.myc:144` | `OracleUnavailable` | — | Empirical/Declared |
| `std.testing::FailRecord` | type | `lib/std/testing.myc:150` | `type FailRecord[F] = FRec(F, Binary{64}, Binary{8}, Bytes)` | FailRecord — the structured failure record: FRec(payload, seed, trial, context). The payload (generic F) is the REIFIED counterexample/diff value substituting Rust's Debug-formatted description String (FLAG-testing-3 — the EXPLAIN artifact in value form); the seed makes the failure reproducible (RT3); the trial index locates it; the context Bytes names the op. | Empirical/Declared |
| `std.testing::FailRecord::FRec` | ctor | `lib/std/testing.myc:150` | `FRec(F, Binary{64}, Binary{8}, Bytes)` | — | Empirical/Declared |
| `std.testing::Verdict` | type | `lib/std/testing.myc:155` | `type Verdict[V] = Pass \| Fail(FailRecord[V]) \| Skipped(SkipReason) \| Undetermined(UndetReason)` | Verdict — the outcome sum. Skipped and Undetermined are FIRST-CLASS variants, not flavours of Pass (the honesty crux): the aggregator below keeps their counts distinct, so "green" can never silently include "did not actually check". | Empirical/Declared |
| `std.testing::Verdict::Fail` | ctor | `lib/std/testing.myc:155` | `Fail(FailRecord[V])` | — | Empirical/Declared |
| `std.testing::Verdict::Pass` | ctor | `lib/std/testing.myc:155` | `Pass` | — | Empirical/Declared |
| `std.testing::Verdict::Skipped` | ctor | `lib/std/testing.myc:155` | `Skipped(SkipReason)` | — | Empirical/Declared |
| `std.testing::Verdict::Undetermined` | ctor | `lib/std/testing.myc:155` | `Undetermined(UndetReason)` | — | Empirical/Declared |
| `std.testing::Summary` | type | `lib/std/testing.myc:160` | `type Summary = Counts(Binary{8}, Binary{8}, Binary{8}, Binary{8})` | Summary — the aggregated outcome: Counts(passed, failed, skipped, undetermined). skipped and undetermined are DISTINCT from passed (C1/G2). Counts are Binary{8} (see the width substitution note: add_u refuses at 256 — never a silent wrap). | Empirical/Declared |
| `std.testing::Summary::Counts` | ctor | `lib/std/testing.myc:160` | `Counts(Binary{8}, Binary{8}, Binary{8}, Binary{8})` | — | Empirical/Declared |
| `std.testing::Budget` | type | `lib/std/testing.myc:165` | `type Budget = Trials(Binary{8})` | — | Empirical/Declared |
| `std.testing::Budget::Trials` | ctor | `lib/std/testing.myc:165` | `Trials(Binary{8})` | — | Empirical/Declared |
| `std.testing::GenStep` | type | `lib/std/testing.myc:170` | `type GenStep[G] = Stepped(Option[G], Binary{64})` | — | Empirical/Declared |
| `std.testing::GenStep::Stepped` | ctor | `lib/std/testing.myc:170` | `Stepped(Option[G], Binary{64})` | — | Empirical/Declared |
| `std.testing::GoldenBaseline` | type | `lib/std/testing.myc:175` | `type GoldenBaseline[S] = GB(Binary{8}, S)` | — | Empirical/Declared |
| `std.testing::GoldenBaseline::GB` | ctor | `lib/std/testing.myc:175` | `GB(Binary{8}, S)` | — | Empirical/Declared |
| `std.testing::GoldenFail` | type | `lib/std/testing.myc:179` | `type GoldenFail[W] = GFail(Binary{8}, W, W)` | GoldenFail — the reified golden mismatch: GFail(name_code, expected, produced) — the typed substitute for Rust's rendered "expected …, got …; diff: …" description (FLAG-testing-3). | Empirical/Declared |
| `std.testing::GoldenFail::GFail` | ctor | `lib/std/testing.myc:179` | `GFail(Binary{8}, W, W)` | — | Empirical/Declared |
| `std.testing::DiffFail` | type | `lib/std/testing.myc:183` | `type DiffFail[D] = DFail(D, D)` | DiffFail — the reified differential disagreement: DFail(lhs, rhs) — both outputs, inspectable (C3); the input description rides in the FailRecord context. | Empirical/Declared |
| `std.testing::DiffFail::DFail` | ctor | `lib/std/testing.myc:183` | `DFail(D, D)` | — | Empirical/Declared |
| `std.testing::CertMode` | type | `lib/std/testing.myc:187` | `type CertMode = Fast \| Balanced \| Certified` | — | Empirical/Declared |
| `std.testing::CertMode::Balanced` | ctor | `lib/std/testing.myc:187` | `Balanced` | — | Empirical/Declared |
| `std.testing::CertMode::Certified` | ctor | `lib/std/testing.myc:187` | `Certified` | — | Empirical/Declared |
| `std.testing::CertMode::Fast` | ctor | `lib/std/testing.myc:187` | `Fast` | — | Empirical/Declared |
| `std.testing::CertScope` | type | `lib/std/testing.myc:191` | `type CertScope = ScGlobal \| ScPhylum \| ScNodule` | CertScope — the @certification declaration scopes (mycelium_proj::CertScope), least-specific first. | Empirical/Declared |
| `std.testing::CertScope::ScGlobal` | ctor | `lib/std/testing.myc:191` | `ScGlobal` | — | Empirical/Declared |
| `std.testing::CertScope::ScNodule` | ctor | `lib/std/testing.myc:191` | `ScNodule` | — | Empirical/Declared |
| `std.testing::CertScope::ScPhylum` | ctor | `lib/std/testing.myc:191` | `ScPhylum` | — | Empirical/Declared |
| `std.testing::CertDecl` | type | `lib/std/testing.myc:194` | `type CertDecl = Decl(CertScope, CertMode)` | CertDecl — one @certification declaration: Decl(scope, mode). | Empirical/Declared |
| `std.testing::CertDecl::Decl` | ctor | `lib/std/testing.myc:194` | `Decl(CertScope, CertMode)` | — | Empirical/Declared |
| `std.testing::ResolvedMode` | type | `lib/std/testing.myc:198` | `type ResolvedMode = RMode(CertMode, Option[CertScope])` | ResolvedMode — the resolution EXPLAIN: RMode(mode, source). source None = the project default (Fast) or a granular override (above the scope lattice), exactly as in mycelium_proj. | Empirical/Declared |
| `std.testing::ResolvedMode::RMode` | ctor | `lib/std/testing.myc:198` | `RMode(CertMode, Option[CertScope])` | — | Empirical/Declared |
| `std.testing::ModeScope` | type | `lib/std/testing.myc:202` | `type ModeScope = MScope(Bool, Bool, Bool)` | ModeScope — the typed per-mode predicate set: MScope(fast_in, balanced_in, certified_in) in CertMode::ALL order. Guarantee: Declared — a scope is a declaration, not a checked theorem. | Empirical/Declared |
| `std.testing::ModeScope::MScope` | ctor | `lib/std/testing.myc:202` | `MScope(Bool, Bool, Bool)` | — | Empirical/Declared |
| `std.testing::ModeTestConfig` | type | `lib/std/testing.myc:206` | `type ModeTestConfig = MTC(Vec[CertDecl], Option[CertMode])` | ModeTestConfig — MTC(decls, granular): the configurable per-test scope; the granular override wins over all scope tiers (above the lattice). | Empirical/Declared |
| `std.testing::ModeTestConfig::MTC` | ctor | `lib/std/testing.myc:206` | `MTC(Vec[CertDecl], Option[CertMode])` | — | Empirical/Declared |
| `std.testing::ModeVisit` | type | `lib/std/testing.myc:209` | `type ModeVisit = MVisit(Vec[CertMode], Vec[CertMode])` | ModeVisit — MVisit(visited, skipped): the never-silent audit of which tiers ran. | Empirical/Declared |
| `std.testing::ModeVisit::MVisit` | ctor | `lib/std/testing.myc:209` | `MVisit(Vec[CertMode], Vec[CertMode])` | — | Empirical/Declared |
| `std.testing::ModeViolation` | type | `lib/std/testing.myc:214` | `type ModeViolation = NegativeFires(CertMode) \| PositiveAbsent(CertMode)` | ModeViolation — the reified cross-mode assertion failure (FLAG-testing-6): the property fires where the scope says it should not (NegativeFires) or is absent where it must fire (PositiveAbsent) — exactly the two panic directions of the Rust assert. | Empirical/Declared |
| `std.testing::ModeViolation::NegativeFires` | ctor | `lib/std/testing.myc:214` | `NegativeFires(CertMode)` | — | Empirical/Declared |
| `std.testing::ModeViolation::PositiveAbsent` | ctor | `lib/std/testing.myc:214` | `PositiveAbsent(CertMode)` | — | Empirical/Declared |
| `std.testing::FallibilityClass` | type | `lib/std/testing.myc:219` | `type FallibilityClass = FalTotal \| FalVerdict` | — | Empirical/Declared |
| `std.testing::FallibilityClass::FalTotal` | ctor | `lib/std/testing.myc:219` | `FalTotal` | — | Empirical/Declared |
| `std.testing::FallibilityClass::FalVerdict` | ctor | `lib/std/testing.myc:219` | `FalVerdict` | — | Empirical/Declared |
| `std.testing::EffectsClass` | type | `lib/std/testing.myc:223` | `type EffectsClass = EffNone \| EffIo \| EffPerBackend` | EffectsClass — the typed half of the effects column: pure, io-declaring, or per-backend io (the "only golden and differential declare IO" discipline, typed instead of substring-matched). | Empirical/Declared |
| `std.testing::EffectsClass::EffIo` | ctor | `lib/std/testing.myc:223` | `EffIo` | — | Empirical/Declared |
| `std.testing::EffectsClass::EffNone` | ctor | `lib/std/testing.myc:223` | `EffNone` | — | Empirical/Declared |
| `std.testing::EffectsClass::EffPerBackend` | ctor | `lib/std/testing.myc:223` | `EffPerBackend` | — | Empirical/Declared |
| `std.testing::MatrixRow` | type | `lib/std/testing.myc:228` | `type MatrixRow = Row(Bytes, Guarantee, FallibilityClass, Bytes, EffectsClass, Bytes, Bool)` | MatrixRow — Row(op, tag, fallibility_class, fallibility_prose, effects_class, effects_prose, explainable). Row DATA is Declared (hand-transcribed from guarantee_matrix.rs::MATRIX); the structural checks below are Empirical over it (std_testing.rs three-way + live Rust oracle). | Empirical/Declared |
| `std.testing::MatrixRow::Row` | ctor | `lib/std/testing.myc:228` | `Row(Bytes, Guarantee, FallibilityClass, Bytes, EffectsClass, Bytes, Bool)` | — | Empirical/Declared |
| `std.testing::bin_eq` | fn | `lib/std/testing.myc:232` | `fn bin_eq(a: Binary{8}, b: Binary{8}) => Bool` | — | Empirical/Declared |
| `std.testing::bool_and` | fn | `lib/std/testing.myc:236` | `fn bool_and(a: Bool, b: Bool) => Bool` | bool_and / bool_or / bool_not / bool_eq: local total connectives (std.diag precedent). | Empirical/Declared |
| `std.testing::bool_or` | fn | `lib/std/testing.myc:239` | `fn bool_or(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `std.testing::bool_not` | fn | `lib/std/testing.myc:242` | `fn bool_not(a: Bool) => Bool` | — | Empirical/Declared |
| `std.testing::bool_eq` | fn | `lib/std/testing.myc:245` | `fn bool_eq(a: Bool, b: Bool) => Bool` | — | Empirical/Declared |
| `std.testing::bool_to_bin` | fn | `lib/std/testing.myc:249` | `fn bool_to_bin(a: Bool) => Binary{8}` | bool_to_bin: the 0/1 indicator (for scope_count). | Empirical/Declared |
| `std.testing::zero64` | fn | `lib/std/testing.myc:254` | `fn zero64() => Binary{64}` | zero64: the Binary{64} zero (seed/trial placeholder in golden/differential FailRecords — Rust sets seed: 0, trial: 0 there). | Empirical/Declared |
| `std.testing::vec_len` | fn | `lib/std/testing.myc:259` | `fn vec_len[A](xs: Vec[A]) => Binary{8}` | vec_len: the number of elements as Binary{8}; add_u refuses at 256 on all paths — never a silent wrap (G2; the std.collections `len` contract, Empirical). | Empirical/Declared |
| `std.testing::rng_default_seed` | fn | `lib/std/testing.myc:265` | `fn rng_default_seed() => Binary{64}` | — | Empirical/Declared |
| `std.testing::rng_new` | fn | `lib/std/testing.myc:270` | `fn rng_new(seed: Binary{64}) => Binary{64}` | rng_new: construct the initial state from a fixed seed; 0 is promoted (never a silent degenerate state). Exact. | Empirical/Declared |
| `std.testing::rng_next` | fn | `lib/std/testing.myc:276` | `fn rng_next(state: Binary{64}) => Binary{64}` | rng_next: advance the state and return it — Xorshift64 (x ^= x<<13; x ^= x>>7; x ^= x<<17), bit-exact vs Rust's Rng::next_u64 (whose output IS its new state). Exact: deterministic; the same state always yields the same output (C4/RT3). | Empirical/Declared |
| `std.testing::rng_next_u32` | fn | `lib/std/testing.myc:288` | `fn rng_next_u32(state: Binary{64}) => Binary{32}` | rng_next_u32: the high 32 bits of the next draw, as Binary{32} — mirrors Rust's `(next_u64() >> 32) as u32`. The narrow is in-range by construction (the high 32 bits are zero after the shift), so the never-silent width_cast prim narrows Exactly (its second operand is a width witness whose bits are ignored). The caller threads the advanced state via rng_next. | Empirical/Declared |
| `std.testing::budget_new` | fn | `lib/std/testing.myc:300` | `fn budget_new(trials: Binary{8}) => Option[Budget]` | — | Empirical/Declared |
| `std.testing::budget_trials` | fn | `lib/std/testing.myc:304` | `fn budget_trials(b: Budget) => Binary{8}` | budget_trials: the number of trials this budget permits. Exact. | Empirical/Declared |
| `std.testing::budget_default` | fn | `lib/std/testing.myc:308` | `fn budget_default() => Budget` | budget_default: the default budget (100 trials — Budget::DEFAULT). Exact. | Empirical/Declared |
| `std.testing::budget_min` | fn | `lib/std/testing.myc:312` | `fn budget_min() => Budget` | budget_min: the minimum budget (1 trial — Budget::MIN). Exact. | Empirical/Declared |
| `std.testing::find_failing` | fn | `lib/std/testing.myc:317` | `fn find_failing[T](candidates: Vec[T], prop: T => Bool) => Option[T]` | — | Empirical/Declared |
| `std.testing::shrink_loop` | fn | `lib/std/testing.myc:325` | `fn shrink_loop[T](best: T, candidates: Vec[T], shrink: T => Vec[T], prop: T => Bool, steps: Binary{16}) => T` | shrink_loop: descend through shrink candidates keeping the smallest still-failing value; the step count is bounded (Rust's literal 1000, at Binary{16}) so shrinking itself is bounded (C6). | Empirical/Declared |
| `std.testing::shrink_to_minimal` | fn | `lib/std/testing.myc:342` | `fn shrink_to_minimal[T](initial: T, shrink: T => Vec[T], prop: T => Bool) => T` | shrink_to_minimal: shrink a failing value to a minimal counterexample (spec §3). The result is the REIFIED minimal value (FLAG-testing-3 — Rust renders it into the description String). | Empirical/Declared |
| `std.testing::for_all_loop` | fn | `lib/std/testing.myc:348` | `fn for_all_loop[T](gen: Binary{64} => GenStep[T], shrink: T => Vec[T], prop: T => Bool, state: Binary{64}, seed: Binary{64}, remaining: Binary{8}, trial: Binary{8}, generated_any: Bool) => Verdict[T]` | for_all_loop: the trial spine — generate, check, recurse. generated_any distinguishes "budget done, everything passed" (Pass) from "the generator never produced" (Skipped(NeedsRecord) — C1: a property that could not run is reported, never a silent pass). | Empirical/Declared |
| `std.testing::for_all` | fn | `lib/std/testing.myc:390` | `fn for_all[T](gen: Binary{64} => GenStep[T], shrink: T => Vec[T], seed: Binary{64}, budget: Budget, prop: T => Bool) => Verdict[T]` | for_all: run a property over `budget` generated inputs. Returns the first failure (shrunk to a minimal counterexample, with the reproducing seed + trial index), Skipped(NeedsRecord) if the generator cannot produce, or Pass. The gen/shrink fn pair substitutes the Gen<T> trait instance (FLAG-testing-1). Guarantee: Exact as a MECHANISM (the verdict is an exact function of the run); a passing for_all backs an EMPIRICAL claim about the property — never Proven; no op here performs that upgrade (C2/VR-5 — the crux this module exists to prevent). | Empirical/Declared |
| `std.testing::golden` | fn | `lib/std/testing.myc:405` | `fn golden{N}(baseline: Option[GoldenBaseline[Binary{N}]], name: Binary{8}, produced: Binary{N}) => Verdict[GoldenFail[Binary{N}]]` | — | Empirical/Declared |
| `std.testing::diff_compare` | fn | `lib/std/testing.myc:427` | `fn diff_compare{N}(input_desc: Bytes, lhs_out: Binary{N}, rhs_out: Binary{N}) => Verdict[DiffFail[Binary{N}]]` | — | Empirical/Declared |
| `std.testing::differential` | fn | `lib/std/testing.myc:449` | `fn differential{N}(input_desc: Bytes, lhs_available: Bool, lhs: Unit => Binary{N}, rhs_available: Bool, rhs: Unit => Binary{N}) => Verdict[DiffFail[Binary{N}]]` | differential: require lhs() == rhs() through two thunked backends. An unavailable backend is Skipped(BackendUnavailable) — NEVER a silent pass (C1/G2, the differential honesty crux); the thunks are only forced when both backends are available. Exact mechanism; the two-level §8-Q5 bar's tag/EXPLAIN level is the Rust crate's own FLAG-Q5, carried over unresolved. | Empirical/Declared |
| `std.testing::bump_pass` | fn | `lib/std/testing.myc:464` | `fn bump_pass(s: Summary) => Summary` | — | Empirical/Declared |
| `std.testing::bump_fail` | fn | `lib/std/testing.myc:467` | `fn bump_fail(s: Summary) => Summary` | — | Empirical/Declared |
| `std.testing::bump_skipped` | fn | `lib/std/testing.myc:470` | `fn bump_skipped(s: Summary) => Summary` | — | Empirical/Declared |
| `std.testing::bump_undetermined` | fn | `lib/std/testing.myc:473` | `fn bump_undetermined(s: Summary) => Summary` | — | Empirical/Declared |
| `std.testing::summarize` | fn | `lib/std/testing.myc:479` | `fn summarize[V](vs: Vec[Verdict[V]]) => Summary` | summarize: aggregate verdicts keeping Skipped/Undetermined counts DISTINCT from Pass — "green" can never silently include "did not actually check" (C1/G2). Exact (a total function over verdicts). | Empirical/Declared |
| `std.testing::is_green` | fn | `lib/std/testing.myc:493` | `fn is_green(s: Summary) => Bool` | is_green: True iff there are no failures. Skipped/undetermined counts are SURFACED in the Summary (the caller can inspect them), not hidden and not treated as failures — treating "could not run" as "failed" would itself violate C1 (the Rust doc's exact reasoning). Exact. | Empirical/Declared |
| `std.testing::summary_passed` | fn | `lib/std/testing.myc:497` | `fn summary_passed(s: Summary) => Binary{8}` | summary field accessors (the .myc idiom for Rust's named struct fields). Exact. | Empirical/Declared |
| `std.testing::summary_failed` | fn | `lib/std/testing.myc:500` | `fn summary_failed(s: Summary) => Binary{8}` | — | Empirical/Declared |
| `std.testing::summary_skipped` | fn | `lib/std/testing.myc:503` | `fn summary_skipped(s: Summary) => Binary{8}` | — | Empirical/Declared |
| `std.testing::summary_undetermined` | fn | `lib/std/testing.myc:506` | `fn summary_undetermined(s: Summary) => Binary{8}` | — | Empirical/Declared |
| `std.testing::summary_total` | fn | `lib/std/testing.myc:512` | `fn summary_total(s: Summary) => Binary{8}` | summary_total: the total verdict count. Substitution note: Rust saturates (saturating_add); add_u instead REFUSES past 255 on all paths — a louder, never-silent overflow discipline (G2), documented rather than hidden. | Empirical/Declared |
| `std.testing::mode_code` | fn | `lib/std/testing.myc:518` | `fn mode_code(m: CertMode) => Binary{8}` | — | Empirical/Declared |
| `std.testing::mode_eq` | fn | `lib/std/testing.myc:522` | `fn mode_eq(a: CertMode, b: CertMode) => Bool` | mode_eq: ADT equality via the injective code map. Exact. | Empirical/Declared |
| `std.testing::modes_all` | fn | `lib/std/testing.myc:526` | `fn modes_all() => Vec[CertMode]` | modes_all: CertMode::ALL — every tier, weakest first. Exact. | Empirical/Declared |
| `std.testing::scope_specificity` | fn | `lib/std/testing.myc:530` | `fn scope_specificity(sc: CertScope) => Binary{8}` | scope_specificity: mirrors CertScope::specificity (Global 0 < Phylum 1 < Nodule 2). Exact. | Empirical/Declared |
| `std.testing::scope_all_modes` | fn | `lib/std/testing.myc:537` | `fn scope_all_modes() => ModeScope` | — | Empirical/Declared |
| `std.testing::scope_fast_only` | fn | `lib/std/testing.myc:540` | `fn scope_fast_only() => ModeScope` | — | Empirical/Declared |
| `std.testing::scope_non_fast` | fn | `lib/std/testing.myc:543` | `fn scope_non_fast() => ModeScope` | — | Empirical/Declared |
| `std.testing::scope_certified_only` | fn | `lib/std/testing.myc:546` | `fn scope_certified_only() => ModeScope` | — | Empirical/Declared |
| `std.testing::scope_emit_modes` | fn | `lib/std/testing.myc:550` | `fn scope_emit_modes() => ModeScope` | scope_emit_modes: alias for scope_non_fast (the certificate-emitting modes — RFC-0034 §5). | Empirical/Declared |
| `std.testing::scope_balanced_only` | fn | `lib/std/testing.myc:553` | `fn scope_balanced_only() => ModeScope` | — | Empirical/Declared |
| `std.testing::scope_contains` | fn | `lib/std/testing.myc:557` | `fn scope_contains(s: ModeScope, m: CertMode) => Bool` | scope_contains: True iff the mode is in scope. Exact (total, match-defined). | Empirical/Declared |
| `std.testing::scope_filter` | fn | `lib/std/testing.myc:561` | `fn scope_filter(s: ModeScope, ms: Vec[CertMode], want: Bool) => Vec[CertMode]` | scope_filter: the modes whose membership equals `want`, in CertMode::ALL order. Exact. | Empirical/Declared |
| `std.testing::modes_in_scope` | fn | `lib/std/testing.myc:571` | `fn modes_in_scope(s: ModeScope) => Vec[CertMode]` | modes_in_scope / modes_out_of_scope: the membership split (0..=3 elements each). Exact. | Empirical/Declared |
| `std.testing::modes_out_of_scope` | fn | `lib/std/testing.myc:574` | `fn modes_out_of_scope(s: ModeScope) => Vec[CertMode]` | — | Empirical/Declared |
| `std.testing::scope_count` | fn | `lib/std/testing.myc:578` | `fn scope_count(s: ModeScope) => Binary{8}` | scope_count: the number of modes in scope (0..=3). Exact. | Empirical/Declared |
| `std.testing::scope_is_empty` | fn | `lib/std/testing.myc:582` | `fn scope_is_empty(s: ModeScope) => Bool` | scope_is_empty: an empty scope is surfaced explicitly, not silently ignored (C1/G2). Exact. | Empirical/Declared |
| `std.testing::scope_union` | fn | `lib/std/testing.myc:586` | `fn scope_union(a: ModeScope, b: ModeScope) => ModeScope` | scope_union / scope_intersect: composable derived scopes. Exact. | Empirical/Declared |
| `std.testing::scope_intersect` | fn | `lib/std/testing.myc:594` | `fn scope_intersect(a: ModeScope, b: ModeScope) => ModeScope` | — | Empirical/Declared |
| `std.testing::from_resolved_mode` | fn | `lib/std/testing.myc:604` | `fn from_resolved_mode(r: ResolvedMode) => ModeScope` | from_resolved_mode: the resolver→scope bridge — "the modes at or above the resolved depth": Fast → ALL_MODES, Balanced → NON_FAST, Certified → CERTIFIED_ONLY. Exact. | Empirical/Declared |
| `std.testing::decl_scope` | fn | `lib/std/testing.myc:615` | `fn decl_scope(d: CertDecl) => CertScope` | — | Empirical/Declared |
| `std.testing::decl_mode` | fn | `lib/std/testing.myc:618` | `fn decl_mode(d: CertDecl) => CertMode` | — | Empirical/Declared |
| `std.testing::decl_beats` | fn | `lib/std/testing.myc:624` | `fn decl_beats(new_d: CertDecl, cur: CertDecl) => Bool` | decl_beats: True iff the NEW declaration's specificity is >= the current winner's — the LAST maximal element wins, mirroring Iterator::max_by_key's documented tie behaviour (the resolve_mode fold this port reimplements — FLAG-testing-7). Exact. | Empirical/Declared |
| `std.testing::resolve_fold` | fn | `lib/std/testing.myc:631` | `fn resolve_fold(decls: Vec[CertDecl], acc: Option[CertDecl]) => ResolvedMode` | resolve_fold: the left-to-right max-specificity fold. Exact. | Empirical/Declared |
| `std.testing::resolve_decls` | fn | `lib/std/testing.myc:650` | `fn resolve_decls(decls: Vec[CertDecl]) => ResolvedMode` | resolve_decls: the reimplemented most-specific-wins resolution (mycelium_proj::resolve_mode's observable contract: highest specificity wins; no declaration → the Fast project default with source None). Pinned against the Rust resolver by the live oracle differential (FLAG-testing-7 — the KC-3 shared-resolver property does not carry over). Exact mechanism. | Empirical/Declared |
| `std.testing::config_new` | fn | `lib/std/testing.myc:654` | `fn config_new(decls: Vec[CertDecl]) => ModeTestConfig` | config_new: a config from @certification scope declarations (no granular override). Exact. | Empirical/Declared |
| `std.testing::config_default` | fn | `lib/std/testing.myc:659` | `fn config_default() => ModeTestConfig` | config_default: no declarations → project default Fast → ALL_MODES scope (widening from fast is always safe — the Rust Default impl). Exact. | Empirical/Declared |
| `std.testing::config_with_granular` | fn | `lib/std/testing.myc:664` | `fn config_with_granular(cfg: ModeTestConfig, m: CertMode) => ModeTestConfig` | config_with_granular: add/replace the granular per-test override — wins over all scope tiers. Exact. | Empirical/Declared |
| `std.testing::config_provenance` | fn | `lib/std/testing.myc:670` | `fn config_provenance(cfg: ModeTestConfig) => ResolvedMode` | config_provenance: the resolved mode + its source scope — the EXPLAIN of the scope decision (never ambient, G2). A granular override reports source None (above the lattice), exactly as in Rust. Exact. | Empirical/Declared |
| `std.testing::config_resolve` | fn | `lib/std/testing.myc:675` | `fn config_resolve(cfg: ModeTestConfig) => ModeScope` | config_resolve: the effective ModeScope (provenance → from_resolved_mode — the canonical shared mapping). Exact. | Empirical/Declared |
| `std.testing::visit_visited` | fn | `lib/std/testing.myc:680` | `fn visit_visited(v: ModeVisit) => Vec[CertMode]` | — | Empirical/Declared |
| `std.testing::visit_skipped` | fn | `lib/std/testing.myc:683` | `fn visit_skipped(v: ModeVisit) => Vec[CertMode]` | — | Empirical/Declared |
| `std.testing::visited_all` | fn | `lib/std/testing.myc:687` | `fn visited_all(v: ModeVisit) => Bool` | visited_all: True iff all three modes ran. Exact. | Empirical/Declared |
| `std.testing::mode_list_eq` | fn | `lib/std/testing.myc:691` | `fn mode_list_eq(xs: Vec[CertMode], ys: Vec[CertMode]) => Bool` | mode_list_eq: element-wise Vec[CertMode] equality. Exact. | Empirical/Declared |
| `std.testing::matches_scope` | fn | `lib/std/testing.myc:701` | `fn matches_scope(v: ModeVisit, s: ModeScope) => Bool` | matches_scope: True iff exactly the in-scope modes were visited (no more, no fewer). Exact. | Empirical/Declared |
| `std.testing::map_modes` | fn | `lib/std/testing.myc:707` | `fn map_modes[B](ms: Vec[CertMode], f: CertMode => B) => Vec[B]` | for_each_mode: visit EVERY mode (weakest → strongest) — the pure-map substitution for Rust's FnMut visitor: the per-mode results are RETURNED in order (the visit evidence; never-silent about what ran). There is no way to skip a mode through this fn (use for_each_mode_in). Exact. | Empirical/Declared |
| `std.testing::for_each_mode` | fn | `lib/std/testing.myc:710` | `fn for_each_mode[B](f: CertMode => B) => Vec[B]` | — | Empirical/Declared |
| `std.testing::visit_apply` | fn | `lib/std/testing.myc:716` | `fn visit_apply[B](s: ModeScope, ms: Vec[CertMode], f: CertMode => B) => Vec[CertMode]` | visit_apply: apply `f` per in-scope mode (result computed then discarded — the std.error `inspect` technique; `.myc` has no effect surface, so this preserves "f runs per visited mode"), collecting the visited modes. | Empirical/Declared |
| `std.testing::for_each_mode_in` | fn | `lib/std/testing.myc:729` | `fn for_each_mode_in[B](s: ModeScope, f: CertMode => B) => ModeVisit` | for_each_mode_in: run `f` for each mode IN scope, returning the never-silent visited/skipped audit (the caller can assert matches_scope and inspect the skips — C1/G2). Exact. | Empirical/Declared |
| `std.testing::scope_check_walk` | fn | `lib/std/testing.myc:735` | `fn scope_check_walk(s: ModeScope, pred: CertMode => Bool, ms: Vec[CertMode], neg_only: Bool) => Option[ModeViolation]` | — | Empirical/Declared |
| `std.testing::mode_scope_violation` | fn | `lib/std/testing.myc:761` | `fn mode_scope_violation(s: ModeScope, pred: CertMode => Bool) => Option[ModeViolation]` | mode_scope_violation: the decision core of assert_mode_scope — the FIRST violation of either direction (the property fires outside scope, or is absent inside it), reified; None = the assertion holds. The panicking wrapper awaits a host refusal primitive (FLAG-testing-6). Exact. | Empirical/Declared |
| `std.testing::mode_negative_violation` | fn | `lib/std/testing.myc:766` | `fn mode_negative_violation(s: ModeScope, pred: CertMode => Bool) => Option[ModeViolation]` | mode_negative_violation: the negative-only half — the property must be ABSENT outside scope (the "invariant fires where it doesn't apply" check). Exact. | Empirical/Declared |
| `std.testing::guarantee_code` | fn | `lib/std/testing.myc:771` | `fn guarantee_code(g: Guarantee) => Binary{8}` | — | Empirical/Declared |
| `std.testing::guarantee_eq` | fn | `lib/std/testing.myc:779` | `fn guarantee_eq(a: Guarantee, b: Guarantee) => Bool` | — | Empirical/Declared |
| `std.testing::row_op` | fn | `lib/std/testing.myc:783` | `fn row_op(r: MatrixRow) => Bytes` | row accessors. Exact. | Empirical/Declared |
| `std.testing::row_tag` | fn | `lib/std/testing.myc:786` | `fn row_tag(r: MatrixRow) => Guarantee` | — | Empirical/Declared |
| `std.testing::row_fallibility_class` | fn | `lib/std/testing.myc:789` | `fn row_fallibility_class(r: MatrixRow) => FallibilityClass` | — | Empirical/Declared |
| `std.testing::row_fallibility` | fn | `lib/std/testing.myc:792` | `fn row_fallibility(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.testing::row_effects_class` | fn | `lib/std/testing.myc:795` | `fn row_effects_class(r: MatrixRow) => EffectsClass` | — | Empirical/Declared |
| `std.testing::row_effects` | fn | `lib/std/testing.myc:798` | `fn row_effects(r: MatrixRow) => Bytes` | — | Empirical/Declared |
| `std.testing::row_explainable` | fn | `lib/std/testing.myc:801` | `fn row_explainable(r: MatrixRow) => Bool` | — | Empirical/Declared |
| `std.testing::is_fal_total` | fn | `lib/std/testing.myc:806` | `fn is_fal_total(f: FallibilityClass) => Bool` | typed column predicates (the ADT analogs of the Rust tests' substring assertions — FLAG-recover precedent: comparisons over typed vocabulary, never over strings). Exact. | Empirical/Declared |
| `std.testing::declares_io` | fn | `lib/std/testing.myc:809` | `fn declares_io(e: EffectsClass) => Bool` | — | Empirical/Declared |
| `std.testing::row_for_all` | fn | `lib/std/testing.myc:814` | `fn row_for_all() => MatrixRow` | — | Empirical/Declared |
| `std.testing::row_golden` | fn | `lib/std/testing.myc:825` | `fn row_golden() => MatrixRow` | — | Empirical/Declared |
| `std.testing::row_differential` | fn | `lib/std/testing.myc:836` | `fn row_differential() => MatrixRow` | — | Empirical/Declared |
| `std.testing::row_summarize` | fn | `lib/std/testing.myc:847` | `fn row_summarize() => MatrixRow` | — | Empirical/Declared |
| `std.testing::row_is_green` | fn | `lib/std/testing.myc:850` | `fn row_is_green() => MatrixRow` | — | Empirical/Declared |
| `std.testing::matrix` | fn | `lib/std/testing.myc:854` | `fn matrix() => Vec[MatrixRow]` | matrix: the full 5-row table, in guarantee_matrix.rs::MATRIX order. | Empirical/Declared |
| `std.testing::all_rows_exact` | fn | `lib/std/testing.myc:860` | `fn all_rows_exact(xs: Vec[MatrixRow]) => Bool` | — | Empirical/Declared |
| `std.testing::all_rows_explainable` | fn | `lib/std/testing.myc:867` | `fn all_rows_explainable(xs: Vec[MatrixRow]) => Bool` | all_rows_explainable: every op surfaces an EXPLAIN artifact (C3/G11/SC-3 — no black boxes). | Empirical/Declared |
| `std.testing::only_golden_and_differential_declare_io` | fn | `lib/std/testing.myc:875` | `fn only_golden_and_differential_declare_io() => Bool` | only_golden_and_differential_declare_io: the spec §4 effects discipline, read off the typed column (no substring matching). | Empirical/Declared |
| `std.testing::aggregator_rows_are_total` | fn | `lib/std/testing.myc:883` | `fn aggregator_rows_are_total() => Bool` | aggregator_rows_are_total: summarize/is_green are total functions over verdicts (spec §4). | Empirical/Declared |

### std.text

| Symbol | Kind | File:Line | Signature | Summary | Tag |
|---|---|---|---|---|---|
| `std.text` | nodule | `lib/std/text.myc:8` | `nodule std.text` | Self-hosted UTF-8 utilities: byte length, slice/concat over the kernel Bytes, ASCII/continuation classification, bounds-checked byte access, full multi-byte UTF-8 codepoint decode (1/2/3/4-byte) WITH validity rejection (overlong / surrogate / >U+10FFFF — M-717 complete). | Empirical/Declared |
| `std.text::Option` | type | `lib/std/text.myc:56` | `type Option[A] = Some(A) \| None` | Scope (cumulative): byte_len, is_ascii_byte, byte_at, full decode_one (1/2/3/4-byte), and — added by DN-43/M-799 — `slice`/`concat` over the kernel `Bytes`. All operations that execute three-way are honest about their basis. ── Local Option (redeclared; no cross-leaf import) ────────────────────────────────────────────── Option<A>: the never-silent optional value. Redeclared here (not imported from std.option) to keep this nodule self-contained per worktree discipline. Semantics identical to std.option. | Empirical/Declared |
| `std.text::Option::None` | ctor | `lib/std/text.myc:56` | `None` | — | Empirical/Declared |
| `std.text::Option::Some` | ctor | `lib/std/text.myc:56` | `Some(A)` | — | Empirical/Declared |
| `std.text::Result` | type | `lib/std/text.myc:60` | `type Result[A, E] = Ok(A) \| Err(E)` | — | Empirical/Declared |
| `std.text::Result::Err` | ctor | `lib/std/text.myc:60` | `Err(E)` | — | Empirical/Declared |
| `std.text::Result::Ok` | ctor | `lib/std/text.myc:60` | `Ok(A)` | — | Empirical/Declared |
| `std.text::Pair` | type | `lib/std/text.myc:66` | `type Pair[A, B] = Pr(A, B)` | — | Empirical/Declared |
| `std.text::Pair::Pr` | ctor | `lib/std/text.myc:66` | `Pr(A, B)` | — | Empirical/Declared |
| `std.text::Utf8Error` | type | `lib/std/text.myc:79` | `type Utf8Error = Invalid(Binary{8}) \| Overlong(Binary{8}) \| Surrogate(Binary{8}) \| TooLarge(Binary{8})` | — | Empirical/Declared |
| `std.text::Utf8Error::Invalid` | ctor | `lib/std/text.myc:80` | `Invalid(Binary{8})` | — | Empirical/Declared |
| `std.text::Utf8Error::Overlong` | ctor | `lib/std/text.myc:81` | `Overlong(Binary{8})` | — | Empirical/Declared |
| `std.text::Utf8Error::Surrogate` | ctor | `lib/std/text.myc:82` | `Surrogate(Binary{8})` | — | Empirical/Declared |
| `std.text::Utf8Error::TooLarge` | ctor | `lib/std/text.myc:83` | `TooLarge(Binary{8})` | — | Empirical/Declared |
| `std.text::Bytes8` | type | `lib/std/text.myc:89` | `type Bytes8 = BNil \| BCons(Binary{8}, Bytes8)` | — | Empirical/Declared |
| `std.text::Bytes8::BCons` | ctor | `lib/std/text.myc:89` | `BCons(Binary{8}, Bytes8)` | — | Empirical/Declared |
| `std.text::Bytes8::BNil` | ctor | `lib/std/text.myc:89` | `BNil` | — | Empirical/Declared |
| `std.text::byte_len` | fn | `lib/std/text.myc:94` | `fn byte_len(b: Bytes) => Binary{32}` | — | Empirical/Declared |
| `std.text::concat` | fn | `lib/std/text.myc:101` | `fn concat(b1: Bytes, b2: Bytes) => Bytes` | — | Empirical/Declared |
| `std.text::slice` | fn | `lib/std/text.myc:113` | `fn slice(b: Bytes, start: Binary{32}, end: Binary{32}) => Bytes` | Guarantee: Exact over the in-range domain (`start <= end <= len`); the bounds contract is Declared/ never-silent (asserted + exhibited three-way by std_bytes_slice.rs, not machine-proven). | Empirical/Declared |
| `std.text::slice_opt` | fn | `lib/std/text.myc:130` | `fn slice_opt(b: Bytes, start: Binary{32}, end: Binary{32}) => Option[Bytes]` | Guarantee: Declared contract (the bounds predicate is Exact `lt`; the Option wrapping is the never-silent contract). Never-silent (G2): out-of-range/inverted ⇒ explicit `None`. | Empirical/Declared |
| `std.text::is_ascii_byte` | fn | `lib/std/text.myc:141` | `fn is_ascii_byte(x: Binary{8}) => Bool` | — | Empirical/Declared |
| `std.text::is_cont_byte` | fn | `lib/std/text.myc:148` | `fn is_cont_byte(x: Binary{8}) => Bool` | — | Empirical/Declared |
| `std.text::byte_at` | fn | `lib/std/text.myc:164` | `fn byte_at(b: Bytes, i: Binary{8}) => Option[Binary{8}]` | Guarantee: Declared contract (the bounds predicate is Exact `lt`/`width_cast`; the Option wrapping is the never-silent contract). Never-silent (G2): an out-of-range index is an explicit `None`, never a kernel refusal leaking through and never a silent wrap. This closes FLAG-text-1 (wave-n1). | Empirical/Declared |
| `std.text::widen8` | fn | `lib/std/text.myc:173` | `fn widen8(x: Binary{8}) => Binary{32}` | — | Empirical/Declared |
| `std.text::dbl` | fn | `lib/std/text.myc:180` | `fn dbl(v: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `std.text::shl6` | fn | `lib/std/text.myc:183` | `fn shl6(v: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `std.text::shl12` | fn | `lib/std/text.myc:186` | `fn shl12(v: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `std.text::shl18` | fn | `lib/std/text.myc:189` | `fn shl18(v: Binary{32}) => Binary{32}` | — | Empirical/Declared |
| `std.text::cont_payload` | fn | `lib/std/text.myc:195` | `fn cont_payload(c: Binary{8}) => Binary{32}` | — | Empirical/Declared |
| `std.text::reject_two` | fn | `lib/std/text.myc:211` | `fn reject_two(cp: Binary{32}, lead: Binary{8}) => Result[Pair[Binary{32}, Binary{8}], Utf8Error]` | reject_two/three/four take the ASSEMBLED codepoint `cp` (Binary{32}) and the sequence's lead byte, and return Ok(Pr(cp, width)) iff `cp` is the CANONICAL well-formed encoding for that byte width — else a never-silent Err carrying the lead byte (G2). They are the RFC-3629 well-formedness layer that the structural decode_two/three/four delegate their final Ok to. Comparisons are Exact `lt` at the common Binary{32} width; the bound constants are written as 32-bit BINARY literals (a `0x…` literal is a `Bytes` value in this lexer, not a Binary{32} number — so the thresholds use `0b…` explicitly): 0x80 = min 2-byte · 0x800 = min 3-byte · 0x10000 = min 4-byte (below ⇒ OVERLONG) · 0xD800..0xE000 = the surrogate gap (SURROGATE) · 0x10FFFF = the Unicode ceiling (above ⇒ TOOLARGE). Guarantee: Declared (the Result wrapping is the never-silent contract; the predicate is Exact `lt`). 2-byte: valid iff cp >= 0x80 (a 1-byte value here would be overlong). The 2-byte max (0x7FF) is below both the surrogate gap and the ceiling, so only the overlong check applies. (0x80 = 1<<7.) | Empirical/Declared |
| `std.text::reject_three` | fn | `lib/std/text.myc:220` | `fn reject_three(cp: Binary{32}, lead: Binary{8}) => Result[Pair[Binary{32}, Binary{8}], Utf8Error]` | 3-byte: valid iff cp >= 0x800 (else overlong) AND cp not in 0xD800..0xDFFF (else a surrogate — not a Unicode scalar value). The 3-byte max (0xFFFF) is below the ceiling, so no too-large check. (0x800 = 1<<11; 0xD800 = surrogate-gap low; 0xE000 = surrogate-gap high+1.) | Empirical/Declared |
| `std.text::reject_four` | fn | `lib/std/text.myc:235` | `fn reject_four(cp: Binary{32}, lead: Binary{8}) => Result[Pair[Binary{32}, Binary{8}], Utf8Error]` | 4-byte: valid iff cp >= 0x10000 (else overlong) AND cp <= 0x10FFFF (else above the Unicode ceiling). The 4-byte min (0x10000) is above the surrogate gap, so no surrogate check. (0x10000 = 1<<16; 0x10FFFF = the maximum Unicode scalar value.) | Empirical/Declared |
| `std.text::decode_ascii` | fn | `lib/std/text.myc:254` | `fn decode_ascii(b: Bytes, i: Binary{8}) => Result[Binary{8}, Utf8Error]` | Guarantee: Declared (type-level contract; three-way differential agreement Empirical, std_text.rs). | Empirical/Declared |
| `std.text::decode_two` | fn | `lib/std/text.myc:267` | `fn decode_two(b: Bytes, i: Binary{8}, lead: Binary{8}) => Result[Pair[Binary{32}, Binary{8}], Utf8Error]` | 2-byte: lead 0b110xxxxx, cont 0b10yyyyyy yields cp = (lead & 0x1F) << 6 \| (cont & 0x3F). | Empirical/Declared |
| `std.text::decode_three` | fn | `lib/std/text.myc:281` | `fn decode_three(b: Bytes, i: Binary{8}, lead: Binary{8}) => Result[Pair[Binary{32}, Binary{8}], Utf8Error]` | 3-byte: lead 0b1110xxxx, two conts yield cp = (lead & 0x0F) << 12 \| (c1 & 0x3F) << 6 \| (c2 & 0x3F). | Empirical/Declared |
| `std.text::decode_four` | fn | `lib/std/text.myc:309` | `fn decode_four(b: Bytes, i: Binary{8}, lead: Binary{8}) => Result[Pair[Binary{32}, Binary{8}], Utf8Error]` | 4-byte: lead 0b11110xxx, three conts yield cp = (lead & 0x07) << 18 \| (c1 & 0x3F) << 12 \| (c2 & 0x3F) << 6 \| (c3 & 0x3F). | Empirical/Declared |
| `std.text::decode_one` | fn | `lib/std/text.myc:375` | `fn decode_one(b: Bytes, i: Binary{8}) => Result[Pair[Binary{32}, Binary{8}], Utf8Error]` | VALIDITY (M-717 — now CLOSED, the last M-717 remainder): the multi-byte arms reject OVERLONG encodings (a value encodable in fewer bytes), SURROGATE-range codepoints (U+D800–DFFF), and codepoints above U+10FFFF — each a never-silent `Err(Overlong/Surrogate/TooLarge(lead))` via the `reject_two/three/four` gates (the assembled codepoint compared against the per-length minimum and the surrogate/ceiling bounds with `lt` at Binary{32}). So `decode_one` yields only well-formed Unicode scalar values (RFC-3629). Structural malformations (bad lead, missing/short continuation, bad continuation) still surface as `Err(Invalid(byte))`. Boundary values (U+0080, U+0800, U+10000, U+10FFFF) are accepted, not over-rejected (std_text.rs validity tests). | Empirical/Declared |

## Flagged items

Constructs/gaps the heuristic could not (or does not yet) extract (G2: never silently dropped):

*(none)*
