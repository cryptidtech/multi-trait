// SPDX-License-Identifier: Apache-2.0
/// Trait for types that have a null/sentinel value.
///
/// This trait allows multiformats types to define a "null" or sentinel value,
/// similar to Option::None but for types where a discriminated union isn't
/// appropriate. Common examples include null CIDs, null signatures, or
/// default/uninitialized identifiers.
///
/// # Use Cases
///
/// - Defining "zero" values for custom ID types
/// - Creating null/empty multiformats objects
/// - Sentinel values in data structures
/// - Default initialization for resource handles
///
/// # Thread Safety
///
/// This trait is `Send + Sync` safe when implemented on thread-safe types.
///
/// # Examples
///
/// ```rust
/// use multitrait::Null;
///
/// #[derive(Debug, PartialEq)]
/// struct UserId(u64);
///
/// impl Null for UserId {
///     fn null() -> Self {
///         UserId(0)
///     }
///
///     fn is_null(&self) -> bool {
///         self.0 == 0
///     }
/// }
///
/// let null_user = UserId::null();
/// assert!(null_user.is_null());
///
/// let valid_user = UserId(42);
/// assert!(!valid_user.is_null());
/// ```
///
/// # Implementation Guidelines
///
/// The null value should be consistent and deterministic. For a given type:
/// - `T::null()` should always return the same logical value
/// - `T::null().is_null()` should always return `true`
/// - The null value should be a valid instance of the type
pub trait Null {
    /// Create a null/sentinel value of this type.
    ///
    /// This should return a deterministic "null" value that satisfies
    /// `is_null()`. The exact meaning of "null" is type-specific.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::Null;
    ///
    /// # struct ResourceId(u32);
    /// # impl Null for ResourceId {
    /// #     fn null() -> Self { ResourceId(0) }
    /// #     fn is_null(&self) -> bool { self.0 == 0 }
    /// # }
    /// let null_id = ResourceId::null();
    /// assert!(null_id.is_null());
    /// ```
    fn null() -> Self;

    /// Check if this value is the null value.
    ///
    /// Returns `true` if this instance represents the null/sentinel value,
    /// `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::Null;
    ///
    /// # struct Counter(i32);
    /// # impl Null for Counter {
    /// #     fn null() -> Self { Counter(0) }
    /// #     fn is_null(&self) -> bool { self.0 == 0 }
    /// # }
    /// let zero = Counter::null();
    /// assert!(zero.is_null());
    ///
    /// let non_zero = Counter(5);
    /// assert!(!non_zero.is_null());
    /// ```
    fn is_null(&self) -> bool;
}

/// Fallible version of [`Null`] for types where null value creation can fail.
///
/// This trait is useful when:
/// - Null value creation requires allocation
/// - Validation is needed during null value construction
/// - Construction can fail in constrained environments
/// - External resources are needed to create the null value
///
/// # Relationship to `Null`
///
/// While [`Null`] provides infallible null value creation, `TryNull` handles
/// cases where construction might fail. If your type's null value is always
/// constructible, prefer implementing [`Null`] instead.
///
/// # Thread Safety
///
/// This trait is `Send + Sync` safe when implemented on thread-safe types
/// with thread-safe error types.
///
/// # Examples
///
/// ```rust
/// use multitrait::TryNull;
///
/// #[derive(Debug)]
/// struct BufferId(Vec<u8>);
///
/// impl TryNull for BufferId {
///     type Error = std::collections::TryReserveError;
///
///     fn try_null() -> Result<Self, Self::Error> {
///         let mut vec = Vec::new();
///         vec.try_reserve(1)?;
///         vec.push(0);
///         Ok(BufferId(vec))
///     }
///
///     fn is_null(&self) -> bool {
///         self.0.len() == 1 && self.0[0] == 0
///     }
/// }
///
/// match BufferId::try_null() {
///     Ok(null_buf) => assert!(null_buf.is_null()),
///     Err(e) => eprintln!("Failed to create null buffer: {}", e),
/// }
/// ```
///
/// # Implementation Guidelines
///
/// - The error type should describe why null creation failed
/// - If `try_null()` succeeds, the result should satisfy `is_null()`
/// - The null value should be consistent across successful calls
pub trait TryNull: Sized {
    /// The error type returned when null value creation fails.
    type Error;

    /// Try to construct a null/sentinel value of this type.
    ///
    /// Returns `Ok` with a null value if successful, or `Err` if null value
    /// creation failed (e.g., due to allocation failure or validation error).
    ///
    /// # Errors
    ///
    /// Returns an error if null value construction fails. The exact error
    /// conditions are type-specific.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::TryNull;
    ///
    /// # struct ValidatedId(String);
    /// # impl TryNull for ValidatedId {
    /// #     type Error = &'static str;
    /// #     fn try_null() -> Result<Self, Self::Error> {
    /// #         Ok(ValidatedId(String::new()))
    /// #     }
    /// #     fn is_null(&self) -> bool { self.0.is_empty() }
    /// # }
    /// let null_id = ValidatedId::try_null()?;
    /// assert!(null_id.is_null());
    /// # Ok::<(), &'static str>(())
    /// ```
    fn try_null() -> Result<Self, Self::Error>;

    /// Check if this value is the null value.
    ///
    /// Returns `true` if this instance represents the null/sentinel value,
    /// `false` otherwise. This method is identical to [`Null::is_null`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::TryNull;
    ///
    /// # struct OptionalData(Option<Vec<u8>>);
    /// # impl TryNull for OptionalData {
    /// #     type Error = std::string::String;
    /// #     fn try_null() -> Result<Self, Self::Error> {
    /// #         Ok(OptionalData(None))
    /// #     }
    /// #     fn is_null(&self) -> bool { self.0.is_none() }
    /// # }
    /// let data = OptionalData::try_null().unwrap();
    /// assert!(data.is_null());
    /// ```
    fn is_null(&self) -> bool;
}
