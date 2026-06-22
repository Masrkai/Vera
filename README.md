
<p align="center">
  <img src="./assets/Vera.svg" alt="Vera Logo" width="128">
</p>

# Vera

**A simple tool for protecting your privacy**

*Project initialized on July 14, 2025, by @Masrkai*
> Note: this repo doesn't reflect the date because it was rebased alot previously it was named as `EXIF-Dumper` and was wriiten in python see [here](./docs/chouce.md) the iterations i had to go through to get to this point

---

## Why This Project Exists

This project was born from research into OSINT (Open Source Intelligence) techniques, specifically how images can be weaponized against users through embedded EXIF metadata. Most people don't realize that their photos contain a treasure trove of sensitive information that can be exploited by malicious actors.

### What EXIF Data Can Reveal

When you share an image, you might unknowingly be sharing:

- **GPS coordinates** - Exact location where the photo was taken
- **Device information** - Phone brand, model, and specifications
- **Camera settings** - ISO, aperture, shutter speed, flash usage
- **Timestamps** - Precise date and time of capture
- **Software details** - Apps used to edit or process the image

This metadata creates a detailed digital fingerprint that makes tracking you significantly easier. An attacker can determine what device you're using, where you've been, and build a profile of your daily patterns—all from seemingly innocent photos.

---

## The Solution

Vera provides a straightforward way to sanitize your images by editing or deleting EXIF data as needed. This tool ensures your personal information isn't inadvertently shared when you post photos online.

### Key Features

- **Metadata removal** - Strip all EXIF data from images
- **Batch processing** - Handle multiple images at once (deleting their metadata only)
- **Privacy-focused**  - No data collection or cloud processing

---

## Platform Support

**Currently supported:**

- NixOS Linux

**Planned (still not approached):**

- Mobile platforms

---

## "But Platforms Already Strip EXIF Data"

**This is a dangerous misconception.**

While some platforms like Instagram and Facebook claim to remove EXIF data, consider these critical points:

### Can You Trust Big Tech?

- These platforms often **extract and store** your location data before "removing" it
- They use this information for targeted advertising and user profiling
- Your data becomes part of their business model, not truly deleted

### The Reality of "Stripped" Data

Even when platforms remove EXIF data from public posts, they may:

- Keep the original files with full metadata on their servers
- Use the geolocation data for their own analytics
- Share this information with third parties or government agencies

**Taking control of your data *before* uploading is the only way to ensure true privacy.**

---

## "I Have Nothing to Hide"

This argument fundamentally misunderstands the nature and purpose of privacy rights. It reduces privacy to a binary state of innocence versus guilt, implying that only those with something illicit to conceal have a legitimate claim to secrecy. This is a dangerous oversimplification.

**Privacy isn't about hiding wrongdoing; it is about maintaining autonomy, dignity, and control over your personal information.** It is the right to decide what parts of your life are shared, with whom, and under what circumstances. Just as we close the door when using the restroom or draw the curtains at night not because we are committing crimes, but because we value intimacy and contextual integrity digital privacy serves the same function in the modern age.

Consider this analogy: Would you be comfortable with the following scenarios?

- **A camera in your bedroom streaming live to the outside world.** Even if you are simply sleeping, reading, or changing clothes, the lack of consent and the potential for misuse, judgment, or exploitation violates your fundamental sense of safety and bodily autonomy. You have "nothing illegal" happening, yet the intrusion is unacceptable.

- **A corporation using publicly available data to undermine your legal rights.** Imagine you are suing a company for wrongful termination or breach of contract. If they can aggregate trivial data points your shopping habits, location history, or social media likesvto construct a misleading narrative about your character or credibility, they gain an unfair advantage. Privacy ensures that your digital footprint cannot be weaponized against you in contexts where you have genuine rights they have violated.

- **Unwanted surveillance by stalkers, harassers, or malicious actors.** Privacy protections are not just about defending against state overreach or corporate greed; they are also a shield against interpersonal harm. Without robust privacy norms, individuals become vulnerable to doxxing, stalking, and targeted harassment. The "nothing to hide" argument offers no protection to victims of abuse who rely on obscurity and data control for their physical safety.

Of course not. The discomfort you feel in these analogies stems from the loss of agency, not the presence of guilt. The same principle applies to your digital footprint. In an era where data is permanently recorded, easily aggregated, and algorithmically analyzed, the power imbalance between the individual and the observer is vast.

Furthermore, context matters. Information that is harmless in one setting can be damaging in another. A medical condition, a political affiliation, or a financial struggle may be irrelevant to your employer, your insurance provider, or the general public, yet without privacy controls, these details can lead to discrimination, higher premiums, or social stigma.

Ultimately, asserting that you have "nothing to hide" is akin to saying you have "nothing to say" regarding free speech. It ignores the chilling effect that constant surveillance has on behavior, creativity, and dissent. When people know they are being watched, they self-censor. They avoid exploring controversial topics, seeking sensitive medical advice, or associating with marginalized groups. Privacy is the bedrock of a free society because it allows individuals to think, explore, and exist without the fear of perpetual judgment or repercussion.

### Why Privacy Matters

- **Personal safety** - Preventing stalking, harassment, or physical harm
- **Professional protection** - Avoiding discrimination or career damage
- **Family security** - Protecting loved ones from unwanted attention
- **Future freedom** - Ensuring today's data can't be used against you tomorrow

Privacy is a fundamental human right, not a privilege for those with "something to hide."

---

## Getting Started

Ready to take control of your image privacy? Check out the installation guide (comming soon) and start protecting your digital footprint today.

*Remember: Your privacy is your responsibility. Don't leave it to chance.*
