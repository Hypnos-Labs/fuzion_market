# ||| Changelog |||

# [0.2.0]

### Added/Removed
<details>
	<summary>**Removed:** state `Config`</summary>
    <ul>
    <li>Removed `Config` from state as it was never used, and a contract level admin is not required </li>
    </ul>
</details>
<details>
	<summary>**Added:** `FeeDenom` to State, used for ___soon___</summary>
</details>
<details>
	<summary>**Added:** New `FeeCalc` mechanism to support `FeeDenom`</summary>
    <ul>
    <li>Added mechanism to both Listing & Bucket</li>
    <li>Fees are now calculated when swap is executed, instead of when withdrawn</li>
    </ul>
</details></br>

### Changed
<details>
	<summary>Error if GenericBalance not normalized</summary>
    <ul>
    <li>Return an error if a GenericBalance in ask is not normalized (contains 0 values or duplicate native/cw20 denoms), instead of using the cw-utils Normalize method manually</li>
    <li> Think returning an error to let user know this isn't an option is better than making the decision for them</li>
    </ul>
</details>
<details>
	<summary>Combine `Refund` & `Remove` into 1 new function `Delete` </summary>
</details>
<details>
	<summary>Moved imports by file to `lib.rs` </summary>
</details></br>

### Bug Fixes
<details>
	<summary>**`GenericBalance` equality checks**</summary>
    <ul>
    <li>Previously, `GenericBalance`'s were compared in the buy_listing function via the != / == operators</li>
    <li>This type of comparison was "sufficient", in that there would never be a false positive == / false negative !=</li>
    <li> But, it would result in false negative == / false positive != in cases where items in the `GenericBalance`'s fields were not identically sorted </li>
    <li>**FIX:** Add a custom `GenericBalance` comparison function that does not result in false negative == / false positive !=, regardless of if the Vecs are identically sorted</li>
    </ul>
</details>
<details>
	<summary>Replace arbitrary Listing IDs</summary>
    <ul>
    <li> Replaced user choosen `listing_id: String` with a state incremented `listing_id: u64`</li>
    <li> The idea is to prevent backrunning/replacement attacks, where a malicious user deletes a listing with a large for_sale amount, then immediately replaces it with a much lower for_sale amount, with the same `listing_id` as the first</li>
    <li> Latency in UI updates could have caused this to be an issue to users</li>
    </ul>
</details>
<details>
	<summary>Replace arbitrary Bucket IDs</summary>
    <ul>
    <li> Replaced user choosen `bucket_id: String` with a state incremented `bucket_id: u64`</li>
    <li> This could cause issues where previous buckets end up getting overwritten in state with new buckets using the same bucket_id</li>
    </ul>
</details></br>

### Misc
-Added unit tests for new `GenericBalance` compare function  
-Created changelog  
-Add doc comments

### To Do
[ ] Add/Modify `integration_tests` to reflect new changes  
[ ] Generally clean up `integration_tests`  
[ ] Add/Modify `e2e` tests to reflect new changes (lots of work needed here)  
[ ] Add Pupmos `MsgFundCommunityPool` implementation  
[ ] Remove unneeded Error variants  
    

