use crate::prelude::*;
// package me.ag2s.epublib.domain;


/**
 * A relator denotes which role a certain individual had in the creation/modification of the ebook.
 *
 * Examples are 'creator', 'blurb writer', etc.
 *
 * This is contains the complete Library of Concress relator list.
 *
 * @see <a href="http://www.loc.gov/marc/relators/relaterm.html">MARC Code List for Relators</a>
 *
 * @author paul
 */
pub enum Relator {

  /**
   * Use for a person or organization who principally exhibits acting skills in a musical or dramatic presentation or entertainment.
   */
  ACTOR,

  /**
   * Use for a person or organization who 1) reworks a musical composition, usually for a different medium, or 2) rewrites novels or stories for motion pictures or other audiovisual medium.
   */
  ADAPTER,

  /**
   * Use for a person or organization that reviews, examines and interprets data or information in a specific area.
   */
  ANALYST,

  /**
   * Use for a person or organization who draws the two-dimensional figures, manipulates the three dimensional objects and/or also programs the computer to move objects and images for the purpose of animated film processing. Animation cameras, stands, celluloid screens, transparencies and inks are some of the tools of the animator.
   */
  ANIMATOR,

  /**
   * Use for a person who writes manuscript annotations on a printed item.
   */
  ANNOTATOR,

  /**
   * Use for a person or organization responsible for the submission of an application or who is named as eligible for the results of the processing of the application (e.g., bestowing of rights, reward, title, position).
   */
  APPLICANT,

  /**
   * Use for a person or organization who designs structures or oversees their construction.
   */
  ARCHITECT,

  /**
   * Use for a person or organization who transcribes a musical composition, usually for a different medium from that of the original; in an arrangement the musical substance remains essentially unchanged.
   */
  ARRANGER,

  /**
   * Use for a person (e.g., a painter or sculptor) who makes copies of works of visual art.
   */
  ART_COPYIST,

  /**
   * Use for a person (e.g., a painter) or organization who conceives, and perhaps also implements, an original graphic design or work of art, if specific codes (e.g., [egr], [etr]) are not desired. For book illustrators, prefer Illustrator [ill].
   */
  ARTIST,

  /**
   * Use for a person responsible for controlling the development of the artistic style of an entire production, including the choice of works to be presented and selection of senior production staff.
   */
  ARTISTIC_DIRECTOR,

  /**
   * Use for a person or organization to whom a license for printing or publishing has been transferred.
   */
  ASSIGNEE,

  /**
   * Use for a person or organization associated with or found in an item or collection, which cannot be determined to be that of a Former owner [fmo] or other designated relator indicative of provenance.
   */
  ASSOCIATED_NAME,

  /**
   * Use for an author, artist, etc., relating him/her to a work for which there is or once was substantial authority for designating that person as author, creator, etc. of the work.
   */
  ATTRIBUTED_NAME,

  /**
   * Use for a person or organization in charge of the estimation and public auctioning of goods, particularly books, artistic works, etc.
   */
  AUCTIONEER,

  /**
   * Use for a person or organization chiefly responsible for the intellectual or artistic content of a work, usually printed text. This term may also be used when more than one person or body bears such responsibility.
   */
  AUTHOR,

  /**
   * Use for a person or organization whose work is largely quoted or extracted in works to which he or she did not contribute directly. Such quotations are found particularly in exhibition catalogs, collections of photographs, etc.
   */
  AUTHOR_IN_QUOTATIONS_OR_TEXT_EXTRACTS,

  /**
   * Use for a person or organization responsible for an afterword, postface, colophon, etc. but who is not the chief author of a work.
   */
  AUTHOR_OF_AFTERWORD_COLOPHON_ETC,

  /**
   * Use for a person or organization responsible for the dialog or spoken commentary for a screenplay or sound recording.
   */
  AUTHOR_OF_DIALOG,

  /**
   * Use for a person or organization responsible for an introduction, preface, foreword, or other critical introductory matter, but who is not the chief author.
   */
  AUTHOR_OF_INTRODUCTION_ETC,

  /**
   * Use for a person or organization responsible for a motion picture screenplay, dialog, spoken commentary, etc.
   */
  AUTHOR_OF_SCREENPLAY_ETC,

  /**
   * Use for a person or organization responsible for a work upon which the work represented by the catalog record is based. This may be appropriate for adaptations, sequels, continuations, indexes, etc.
   */
  BIBLIOGRAPHIC_ANTECEDENT,

  /**
   * Use for a person or organization responsible for the binding of printed or manuscript materials.
   */
  BINDER,

  /**
   * Use for a person or organization responsible for the binding design of a book, including the type of binding, the type of materials used, and any decorative aspects of the binding.
   */
  BINDING_DESIGNER,

  /**
   * Use for the named entity responsible for writing a commendation or testimonial for a work, which appears on or within the publication itself, frequently on the back or dust jacket of print publications or on advertising material for all media.
   */
  BLURB_WRITER,

  /**
   * Use for a person or organization responsible for the entire graphic design of a book, including arrangement of type and illustration, choice of materials, and process used.
   */
  BOOK_DESIGNER,

  /**
   * Use for a person or organization responsible for the production of books and other print media, if specific codes (e.g., [bkd], [egr], [tyd], [prt]) are not desired.
   */
  BOOK_PRODUCER,

  /**
   * Use for a person or organization responsible for the design of flexible covers designed for or published with a book, including the type of materials used, and any decorative aspects of the bookjacket.
   */
  BOOKJACKET_DESIGNER,

  /**
   * Use for a person or organization responsible for the design of a book owner's identification label that is most commonly pasted to the inside front cover of a book.
   */
  BOOKPLATE_DESIGNER,

  /**
   * Use for a person or organization who makes books and other bibliographic materials available for purchase. Interest in the materials is primarily lucrative.
   */
  BOOKSELLER,

  /**
   * Use for a person or organization who writes in an artistic hand, usually as a copyist and or engrosser.
   */
  CALLIGRAPHER,

  /**
   * Use for a person or organization responsible for the creation of maps and other cartographic materials.
   */
  CARTOGRAPHER,

  /**
   * Use for a censor, bowdlerizer, expurgator, etc., official or private.
   */
  CENSOR,

  /**
   * Use for a person or organization who composes or arranges dances or other movements (e.g., "master of swords") for a musical or dramatic presentation or entertainment.
   */
  CHOREOGRAPHER,

  /**
   * Use for a person or organization who is in charge of the images captured for a motion picture film. The cinematographer works under the supervision of a director, and may also be referred to as director of photography. Do not confuse with videographer.
   */
  CINEMATOGRAPHER,

  /**
   * Use for a person or organization for whom another person or organization is acting.
   */
  CLIENT,

  /**
   * Use for a person or organization that takes a limited part in the elaboration of a work of another person or organization that brings complements (e.g., appendices, notes) to the work.
   */
  COLLABORATOR,

  /**
   * Use for a person or organization who has brought together material from various sources that has been arranged, described, and cataloged as a collection. A collector is neither the creator of the material nor a person to whom manuscripts in the collection may have been addressed.
   */
  COLLECTOR,

  /**
   * Use for a person or organization responsible for the production of photographic prints from film or other colloid that has ink-receptive and ink-repellent surfaces.
   */
  COLLOTYPER,

  /**
   * Use for the named entity responsible for applying color to drawings, prints, photographs, maps, moving images, etc.
   */
  COLORIST,

  /**
   * Use for a person or organization who provides interpretation, analysis, or a discussion of the subject matter on a recording, motion picture, or other audiovisual medium.
   */
  COMMENTATOR,

  /**
   * Use for a person or organization responsible for the commentary or explanatory notes about a text. For the writer of manuscript annotations in a printed book, use Annotator [ann].
   */
  COMMENTATOR_FOR_WRITTEN_TEXT,

  /**
   * Use for a person or organization who produces a work or publication by selecting and putting together material from the works of various persons or bodies.
   */
  COMPILER,

  /**
   * Use for the party who applies to the courts for redress, usually in an equity proceeding.
   */
  COMPLAINANT,

  /**
   * Use for a complainant who takes an appeal from one court or jurisdiction to another to reverse the judgment, usually in an equity proceeding.
   */
  COMPLAINANT_APPELLANT,

  /**
   * Use for a complainant against whom an appeal is taken from one court or jurisdiction to another to reverse the judgment, usually in an equity proceeding.
   */
  COMPLAINANT_APPELLEE,

  /**
   * Use for a person or organization who creates a musical work, usually a piece of music in manuscript or printed form.
   */
  COMPOSER,

  /**
   * Use for a person or organization responsible for the creation of metal slug, or molds made of other materials, used to produce the text and images in printed matter.
   */
  COMPOSITOR,

  /**
   * Use for a person or organization responsible for the original idea on which a work is based, this includes the scientific author of an audio-visual item and the conceptor of an advertisement.
   */
  CONCEPTOR,

  /**
   * Use for a person who directs a performing group (orchestra, chorus, opera, etc.) in a musical or dramatic presentation or entertainment.
   */
  CONDUCTOR,

  /**
   * Use for the named entity responsible for documenting, preserving, or treating printed or manuscript material, works of art, artifacts, or other media.
   */
  CONSERVATOR,

  /**
   * Use for a person or organization relevant to a resource, who is called upon for professional advice or services in a specialized field of knowledge or training.
   */
  CONSULTANT,

  /**
   * Use for a person or organization relevant to a resource, who is engaged specifically to provide an intellectual overview of a strategic or operational task and by analysis, specification, or instruction, to create or propose a cost-effective course of action or solution.
   */
  CONSULTANT_TO_A_PROJECT,

  /**
   * Use for the party who opposes, resists, or disputes, in a court of law, a claim, decision, result, etc.
   */
  CONTESTANT,

  /**
   * Use for a contestant who takes an appeal from one court of law or jurisdiction to another to reverse the judgment.
   */
  CONTESTANT_APPELLANT,

  /**
   * Use for a contestant against whom an appeal is taken from one court of law or jurisdiction to another to reverse the judgment.
   */
  CONTESTANT_APPELLEE,

  /**
   * Use for the party defending a claim, decision, result, etc. being opposed, resisted, or disputed in a court of law.
   */
  CONTESTEE,

  /**
   * Use for a contestee who takes an appeal from one court or jurisdiction to another to reverse the judgment.
   */
  CONTESTEE_APPELLANT,

  /**
   * Use for a contestee against whom an appeal is taken from one court or jurisdiction to another to reverse the judgment.
   */
  CONTESTEE_APPELLEE,

  /**
   * Use for a person or organization relevant to a resource, who enters into a contract with another person or organization to perform a specific task.
   */
  CONTRACTOR,

  /**
   * Use for a person or organization one whose work has been contributed to a larger work, such as an anthology, serial publication, or other compilation of individual works. Do not use if the sole function in relation to a work is as author, editor, compiler or translator.
   */
  CONTRIBUTOR,

  /**
   * Use for a person or organization listed as a copyright owner at the time of registration. Copyright can be granted or later transferred to another person or organization, at which time the claimant becomes the copyright holder.
   */
  COPYRIGHT_CLAIMANT,

  /**
   * Use for a person or organization to whom copy and legal rights have been granted or transferred for the intellectual content of a work. The copyright holder, although not necessarily the creator of the work, usually has the exclusive right to benefit financially from the sale and use of the work to which the associated copyright protection applies.
   */
  COPYRIGHT_HOLDER,

  /**
   * Use for a person or organization who is a corrector of manuscripts, such as the scriptorium official who corrected the work of a scribe. For printed matter, use Proofreader.
   */
  CORRECTOR,

  /**
   * Use for a person or organization who was either the writer or recipient of a letter or other communication.
   */
  CORRESPONDENT,

  /**
   * Use for a person or organization who designs or makes costumes, fixes hair, etc., for a musical or dramatic presentation or entertainment.
   */
  COSTUME_DESIGNER,

  /**
   * Use for a person or organization responsible for the graphic design of a book cover, album cover, slipcase, box, container, etc. For a person or organization responsible for the graphic design of an entire book, use Book designer; for book jackets, use Bookjacket designer.
   */
  COVER_DESIGNER,

  /**
   * Use for a person or organization responsible for the intellectual or artistic content of a work.
   */
  CREATOR,

  /**
   * Use for a person or organization responsible for conceiving and organizing an exhibition.
   */
  CURATOR_OF_AN_EXHIBITION,

  /**
   * Use for a person or organization who principally exhibits dancing skills in a musical or dramatic presentation or entertainment.
   */
  DANCER,

  /**
   * Use for a person or organization that submits data for inclusion in a database or other collection of data.
   */
  DATA_CONTRIBUTOR,

  /**
   * Use for a person or organization responsible for managing databases or other data sources.
   */
  DATA_MANAGER,

  /**
   * Use for a person or organization to whom a book, manuscript, etc., is dedicated (not the recipient of a gift).
   */
  DEDICATEE,

  /**
   * Use for the author of a dedication, which may be a formal statement or in epistolary or verse form.
   */
  DEDICATOR,

  /**
   * Use for the party defending or denying allegations made in a suit and against whom relief or recovery is sought in the courts, usually in a legal action.
   */
  DEFENDANT,

  /**
   * Use for a defendant who takes an appeal from one court or jurisdiction to another to reverse the judgment, usually in a legal action.
   */
  DEFENDANT_APPELLANT,

  /**
   * Use for a defendant against whom an appeal is taken from one court or jurisdiction to another to reverse the judgment, usually in a legal action.
   */
  DEFENDANT_APPELLEE,

  /**
   * Use for the organization granting a degree for which the thesis or dissertation described was presented.
   */
  DEGREE_GRANTOR,

  /**
   * Use for a person or organization executing technical drawings from others' designs.
   */
  DELINEATOR,

  /**
   * Use for an entity depicted or portrayed in a work, particularly in a work of art.
   */
  DEPICTED,

  /**
   * Use for a person or organization placing material in the physical custody of a library or repository without transferring the legal title.
   */
  DEPOSITOR,

  /**
   * Use for a person or organization responsible for the design if more specific codes (e.g., [bkd], [tyd]) are not desired.
   */
  DESIGNER,

  /**
   * Use for a person or organization who is responsible for the general management of a work or who supervises the production of a performance for stage, screen, or sound recording.
   */
  DIRECTOR,

  /**
   * Use for a person who presents a thesis for a university or higher-level educational degree.
   */
  DISSERTANT,

  /**
   * Use for the name of a place from which a resource, e.g., a serial, is distributed.
   */
  DISTRIBUTION_PLACE,

  /**
   * Use for a person or organization that has exclusive or shared marketing rights for an item.
   */
  DISTRIBUTOR,

  /**
   * Use for a person or organization who is the donor of a book, manuscript, etc., to its present owner. Donors to previous owners are designated as Former owner [fmo] or Inscriber [ins].
   */
  DONOR,

  /**
   * Use for a person or organization who prepares artistic or technical drawings.
   */
  DRAFTSMAN,

  /**
   * Use for a person or organization to which authorship has been dubiously or incorrectly ascribed.
   */
  DUBIOUS_AUTHOR,

  /**
   * Use for a person or organization who prepares for publication a work not primarily his/her own, such as by elucidating text, adding introductory or other critical matter, or technically directing an editorial staff.
   */
  EDITOR,

  /**
   * Use for a person responsible for setting up a lighting rig and focusing the lights for a production, and running the lighting at a performance.
   */
  ELECTRICIAN,

  /**
   * Use for a person or organization who creates a duplicate printing surface by pressure molding and electrodepositing of metal that is then backed up with lead for printing.
   */
  ELECTROTYPER,

  /**
   * Use for a person or organization that is responsible for technical planning and design, particularly with construction.
   */
  ENGINEER,

  /**
   * Use for a person or organization who cuts letters, figures, etc. on a surface, such as a wooden or metal plate, for printing.
   */
  ENGRAVER,

  /**
   * Use for a person or organization who produces text or images for printing by subjecting metal, glass, or some other surface to acid or the corrosive action of some other substance.
   */
  ETCHER,

  /**
   * Use for the name of the place where an event such as a conference or a concert took place.
   */
  EVENT_PLACE,

  /**
   * Use for a person or organization in charge of the description and appraisal of the value of goods, particularly rare items, works of art, etc.
   */
  EXPERT,

  /**
   * Use for a person or organization that executed the facsimile.
   */
  FACSIMILIST,

  /**
   * Use for a person or organization that manages or supervises the work done to collect raw data or do research in an actual setting or environment (typically applies to the natural and social sciences).
   */
  FIELD_DIRECTOR,

  /**
   * Use for a person or organization who is an editor of a motion picture film. This term is used regardless of the medium upon which the motion picture is produced or manufactured (e.g., acetate film, video tape).
   */
  FILM_EDITOR,

  /**
   * Use for a person or organization who is identified as the only party or the party of the first part. In the case of transfer of right, this is the assignor, transferor, licensor, grantor, etc. Multiple parties can be named jointly as the first party
   */
  FIRST_PARTY,

  /**
   * Use for a person or organization who makes or imitates something of value or importance, especially with the intent to defraud.
   */
  FORGER,

  /**
   * Use for a person or organization who owned an item at any time in the past. Includes those to whom the material was once presented. A person or organization giving the item to the present owner is designated as Donor [dnr]
   */
  FORMER_OWNER,

  /**
   * Use for a person or organization that furnished financial support for the production of the work.
   */
  FUNDER,

  /**
   * Use for a person responsible for geographic information system (GIS) development and integration with global positioning system data.
   */
  GEOGRAPHIC_INFORMATION_SPECIALIST,

  /**
   * Use for a person or organization in memory or honor of whom a book, manuscript, etc. is donated.
   */
  HONOREE,

  /**
   * Use for a person who is invited or regularly leads a program (often broadcast) that includes other guests, performers, etc. (e.g., talk show host).
   */
  HOST,

  /**
   * Use for a person or organization responsible for the decoration of a work (especially manuscript material) with precious metals or color, usually with elaborate designs and motifs.
   */
  ILLUMINATOR,

  /**
   * Use for a person or organization who conceives, and perhaps also implements, a design or illustration, usually to accompany a written text.
   */
  ILLUSTRATOR,

  /**
   * Use for a person who signs a presentation statement.
   */
  INSCRIBER,

  /**
   * Use for a person or organization who principally plays an instrument in a musical or dramatic presentation or entertainment.
   */
  INSTRUMENTALIST,

  /**
   * Use for a person or organization who is interviewed at a consultation or meeting, usually by a reporter, pollster, or some other information gathering agent.
   */
  INTERVIEWEE,

  /**
   * Use for a person or organization who acts as a reporter, pollster, or other information gathering agent in a consultation or meeting involving one or more individuals.
   */
  INTERVIEWER,

  /**
   * Use for a person or organization who first produces a particular useful item, or develops a new process for obtaining a known item or result.
   */
  INVENTOR,

  /**
   * Use for an institution that provides scientific analyses of material samples.
   */
  LABORATORY,

  /**
   * Use for a person or organization that manages or supervises work done in a controlled setting or environment.
   */
  LABORATORY_DIRECTOR,

  /**
   * Use for a person or organization whose work involves coordinating the arrangement of existing and proposed land features and structures.
   */
  LANDSCAPE_ARCHITECT,

  /**
   * Use to indicate that a person or organization takes primary responsibility for a particular activity or endeavor. Use with another relator term or code to show the greater importance this person or organization has regarding that particular role. If more than one relator is assigned to a heading, use the Lead relator only if it applies to all the relators.
   */
  LEAD,

  /**
   * Use for a person or organization permitting the temporary use of a book, manuscript, etc., such as for photocopying or microfilming.
   */
  LENDER,

  /**
   * Use for the party who files a libel in an ecclesiastical or admiralty case.
   */
  LIBELANT,

  /**
   * Use for a libelant who takes an appeal from one ecclesiastical court or admiralty to another to reverse the judgment.
   */
  LIBELANT_APPELLANT,

  /**
   * Use for a libelant against whom an appeal is taken from one ecclesiastical court or admiralty to another to reverse the judgment.
   */
  LIBELANT_APPELLEE,

  /**
   * Use for a party against whom a libel has been filed in an ecclesiastical court or admiralty.
   */
  LIBELEE,

  /**
   * Use for a libelee who takes an appeal from one ecclesiastical court or admiralty to another to reverse the judgment.
   */
  LIBELEE_APPELLANT,

  /**
   * Use for a libelee against whom an appeal is taken from one ecclesiastical court or admiralty to another to reverse the judgment.
   */
  LIBELEE_APPELLEE,

  /**
   * Use for a person or organization who is a writer of the text of an opera, oratorio, etc.
   */
  LIBRETTIST,

  /**
   * Use for a person or organization who is an original recipient of the right to print or publish.
   */
  LICENSEE,

  /**
   * Use for person or organization who is a signer of the license, imprimatur, etc.
   */
  LICENSOR,

  /**
   * Use for a person or organization who designs the lighting scheme for a theatrical presentation, entertainment, motion picture, etc.
   */
  LIGHTING_DESIGNER,

  /**
   * Use for a person or organization who prepares the stone or plate for lithographic printing, including a graphic artist creating a design directly on the surface from which printing will be done.
   */
  LITHOGRAPHER,

  /**
   * Use for a person or organization who is a writer of the text of a song.
   */
  LYRICIST,

  /**
   * Use for a person or organization that makes an artifactual work (an object made or modified by one or more persons). Examples of artifactual works include vases, cannons or pieces of furniture.
   */
  MANUFACTURER,

  /**
   * Use for the named entity responsible for marbling paper, cloth, leather, etc. used in construction of a resource.
   */
  MARBLER,

  /**
   * Use for a person or organization performing the coding of SGML, HTML, or XML markup of metadata, text, etc.
   */
  MARKUP_EDITOR,

  /**
   * Use for a person or organization primarily responsible for compiling and maintaining the original description of a metadata set (e.g., geospatial metadata set).
   */
  METADATA_CONTACT,

  /**
   * Use for a person or organization responsible for decorations, illustrations, letters, etc. cut on a metal surface for printing or decoration.
   */
  METAL_ENGRAVER,

  /**
   * Use for a person who leads a program (often broadcast) where topics are discussed, usually with participation of experts in fields related to the discussion.
   */
  MODERATOR,

  /**
   * Use for a person or organization that supervises compliance with the contract and is responsible for the report and controls its distribution. Sometimes referred to as the grantee, or controlling agency.
   */
  MONITOR,

  /**
   * Use for a person who transcribes or copies musical notation
   */
  MUSIC_COPYIST,

  /**
   * Use for a person responsible for basic music decisions about a production, including coordinating the work of the composer, the sound editor, and sound mixers, selecting musicians, and organizing and/or conducting sound for rehearsals and performances.
   */
  MUSICAL_DIRECTOR,

  /**
   * Use for a person or organization who performs music or contributes to the musical content of a work when it is not possible or desirable to identify the function more precisely.
   */
  MUSICIAN,

  /**
   * Use for a person who is a speaker relating the particulars of an act, occurrence, or course of events.
   */
  NARRATOR,

  /**
   * Use for a person or organization responsible for opposing a thesis or dissertation.
   */
  OPPONENT,

  /**
   * Use for a person or organization responsible for organizing a meeting for which an item is the report or proceedings.
   */
  ORGANIZER_OF_MEETING,

  /**
   * Use for a person or organization performing the work, i.e., the name of a person or organization associated with the intellectual content of the work. This category does not include the publisher or personal affiliation, or sponsor except where it is also the corporate author.
   */
  ORIGINATOR,

  /**
   * Use for relator codes from other lists which have no equivalent in the MARC list or for terms which have not been assigned a code.
   */
  OTHER,

  /**
   * Use for a person or organization that currently owns an item or collection.
   */
  OWNER,

  /**
   * Use for a person or organization responsible for the production of paper, usually from wood, cloth, or other fibrous material.
   */
  PAPERMAKER,

  /**
   * Use for a person or organization that applied for a patent.
   */
  PATENT_APPLICANT,

  /**
   * Use for a person or organization that was granted the patent referred to by the item.
   */
  PATENT_HOLDER,

  /**
   * Use for a person or organization responsible for commissioning a work. Usually a patron uses his or her means or influence to support the work of artists, writers, etc. This includes those who commission and pay for individual works.
   */
  PATRON,

  /**
   * Use for a person or organization who exhibits musical or acting skills in a musical or dramatic presentation or entertainment, if specific codes for those functions ([act], [dnc], [itr], [voc], etc.) are not used. If specific codes are used, [prf] is used for a person whose principal skill is not known or specified.
   */
  PERFORMER,

  /**
   * Use for an authority (usually a government agency) that issues permits under which work is accomplished.
   */
  PERMITTING_AGENCY,

  /**
   * Use for a person or organization responsible for taking photographs, whether they are used in their original form or as reproductions.
   */
  PHOTOGRAPHER,

  /**
   * Use for the party who complains or sues in court in a personal action, usually in a legal proceeding.
   */
  PLAINTIFF,

  /**
   * Use for a plaintiff who takes an appeal from one court or jurisdiction to another to reverse the judgment, usually in a legal proceeding.
   */
  PLAINTIFF_APPELLANT,

  /**
   * Use for a plaintiff against whom an appeal is taken from one court or jurisdiction to another to reverse the judgment, usually in a legal proceeding.
   */
  PLAINTIFF_APPELLEE,

  /**
   * Use for a person or organization responsible for the production of plates, usually for the production of printed images and/or text.
   */
  PLATEMAKER,

  /**
   * Use for a person or organization who prints texts, whether from type or plates.
   */
  PRINTER,

  /**
   * Use for a person or organization who prints illustrations from plates.
   */
  PRINTER_OF_PLATES,

  /**
   * Use for a person or organization who makes a relief, intaglio, or planographic printing surface.
   */
  PRINTMAKER,

  /**
   * Use for a person or organization primarily responsible for performing or initiating a process, such as is done with the collection of metadata sets.
   */
  PROCESS_CONTACT,

  /**
   * Use for a person or organization responsible for the making of a motion picture, including business aspects, management of the productions, and the commercial success of the work.
   */
  PRODUCER,

  /**
   * Use for a person responsible for all technical and business matters in a production.
   */
  PRODUCTION_MANAGER,

  /**
   * Use for a person or organization associated with the production (props, lighting, special effects, etc.) of a musical or dramatic presentation or entertainment.
   */
  PRODUCTION_PERSONNEL,

  /**
   * Use for a person or organization responsible for the creation and/or maintenance of computer program design documents, source code, and machine-executable digital files and supporting documentation.
   */
  PROGRAMMER,

  /**
   * Use for a person or organization with primary responsibility for all essential aspects of a project, or that manages a very large project that demands senior level responsibility, or that has overall responsibility for managing projects, or provides overall direction to a project manager.
   */
  PROJECT_DIRECTOR,

  /**
   * Use for a person who corrects printed matter. For manuscripts, use Corrector [crr].
   */
  PROOFREADER,

  /**
   * Use for the name of the place where a resource is published.
   */
  PUBLICATION_PLACE,

  /**
   * Use for a person or organization that makes printed matter, often text, but also printed music, artwork, etc. available to the public.
   */
  PUBLISHER,

  /**
   * Use for a person or organization who presides over the elaboration of a collective work to ensure its coherence or continuity. This includes editors-in-chief, literary editors, editors of series, etc.
   */
  PUBLISHING_DIRECTOR,

  /**
   * Use for a person or organization who manipulates, controls, or directs puppets or marionettes in a musical or dramatic presentation or entertainment.
   */
  PUPPETEER,

  /**
   * Use for a person or organization to whom correspondence is addressed.
   */
  RECIPIENT,

  /**
   * Use for a person or organization who supervises the technical aspects of a sound or video recording session.
   */
  RECORDING_ENGINEER,

  /**
   * Use for a person or organization who writes or develops the framework for an item without being intellectually responsible for its content.
   */
  REDACTOR,

  /**
   * Use for a person or organization who prepares drawings of architectural designs (i.e., renderings) in accurate, representational perspective to show what the project will look like when completed.
   */
  RENDERER,

  /**
   * Use for a person or organization who writes or presents reports of news or current events on air or in print.
   */
  REPORTER,

  /**
   * Use for an agency that hosts data or material culture objects and provides services to promote long term, consistent and shared use of those data or objects.
   */
  REPOSITORY,

  /**
   * Use for a person who directed or managed a research project.
   */
  RESEARCH_TEAM_HEAD,

  /**
   * Use for a person who participated in a research project but whose role did not involve direction or management of it.
   */
  RESEARCH_TEAM_MEMBER,

  /**
   * Use for a person or organization responsible for performing research.
   */
  RESEARCHER,

  /**
   * Use for the party who makes an answer to the courts pursuant to an application for redress, usually in an equity proceeding.
   */
  RESPONDENT,

  /**
   * Use for a respondent who takes an appeal from one court or jurisdiction to another to reverse the judgment, usually in an equity proceeding.
   */
  RESPONDENT_APPELLANT,

  /**
   * Use for a respondent against whom an appeal is taken from one court or jurisdiction to another to reverse the judgment, usually in an equity proceeding.
   */
  RESPONDENT_APPELLEE,

  /**
   * Use for a person or organization legally responsible for the content of the published material.
   */
  RESPONSIBLE_PARTY,

  /**
   * Use for a person or organization, other than the original choreographer or director, responsible for restaging a choreographic or dramatic work and who contributes minimal new content.
   */
  RESTAGER,

  /**
   * Use for a person or organization responsible for the review of a book, motion picture, performance, etc.
   */
  REVIEWER,

  /**
   * Use for a person or organization responsible for parts of a work, often headings or opening parts of a manuscript, that appear in a distinctive color, usually red.
   */
  RUBRICATOR,

  /**
   * Use for a person or organization who is the author of a motion picture screenplay.
   */
  SCENARIST,

  /**
   * Use for a person or organization who brings scientific, pedagogical, or historical competence to the conception and realization on a work, particularly in the case of audio-visual items.
   */
  SCIENTIFIC_ADVISOR,

  /**
   * Use for a person who is an amanuensis and for a writer of manuscripts proper. For a person who makes pen-facsimiles, use Facsimilist [fac].
   */
  SCRIBE,

  /**
   * Use for a person or organization who models or carves figures that are three-dimensional representations.
   */
  SCULPTOR,

  /**
   * Use for a person or organization who is identified as the party of the second part. In the case of transfer of right, this is the assignee, transferee, licensee, grantee, etc. Multiple parties can be named jointly as the second party.
   */
  SECOND_PARTY,

  /**
   * Use for a person or organization who is a recorder, redactor, or other person responsible for expressing the views of a organization.
   */
  SECRETARY,

  /**
   * Use for a person or organization who translates the rough sketches of the art director into actual architectural structures for a theatrical presentation, entertainment, motion picture, etc. Set designers draw the detailed guides and specifications for building the set.
   */
  SET_DESIGNER,

  /**
   * Use for a person whose signature appears without a presentation or other statement indicative of provenance. When there is a presentation statement, use Inscriber [ins].
   */
  SIGNER,

  /**
   * Use for a person or organization who uses his/her/their voice with or without instrumental accompaniment to produce music. A performance may or may not include actual words.
   */
  SINGER,

  /**
   * Use for a person who produces and reproduces the sound score (both live and recorded), the installation of microphones, the setting of sound levels, and the coordination of sources of sound for a production.
   */
  SOUND_DESIGNER,

  /**
   * Use for a person who participates in a program (often broadcast) and makes a formalized contribution or presentation generally prepared in advance.
   */
  SPEAKER,

  /**
   * Use for a person or organization that issued a contract or under the auspices of which a work has been written, printed, published, etc.
   */
  SPONSOR,

  /**
   * Use for a person who is in charge of everything that occurs on a performance stage, and who acts as chief of all crews and assistant to a director during rehearsals.
   */
  STAGE_MANAGER,

  /**
   * Use for an organization responsible for the development or enforcement of a standard.
   */
  STANDARDS_BODY,

  /**
   * Use for a person or organization who creates a new plate for printing by molding or copying another printing surface.
   */
  STEREOTYPER,

  /**
   * Use for a person relaying a story with creative and/or theatrical interpretation.
   */
  STORYTELLER,

  /**
   * Use for a person or organization that supports (by allocating facilities, staff, or other resources) a project, program, meeting, event, data objects, material culture objects, or other entities capable of support.
   */
  SUPPORTING_HOST,

  /**
   * Use for a person or organization who does measurements of tracts of land, etc. to determine location, forms, and boundaries.
   */
  SURVEYOR,

  /**
   * Use for a person who, in the context of a resource, gives instruction in an intellectual subject or demonstrates while teaching physical skills.
   */
  TEACHER,

  /**
   * Use for a person who is ultimately in charge of scenery, props, lights and sound for a production.
   */
  TECHNICAL_DIRECTOR,

  /**
   * Use for a person under whose supervision a degree candidate develops and presents a thesis, mémoire, or text of a dissertation.
   */
  THESIS_ADVISOR,

  /**
   * Use for a person who prepares a handwritten or typewritten copy from original material, including from dictated or orally recorded material. For makers of pen-facsimiles, use Facsimilist [fac].
   */
  TRANSCRIBER,

  /**
   * Use for a person or organization who renders a text from one language into another, or from an older form of a language into the modern form.
   */
  TRANSLATOR,

  /**
   * Use for a person or organization who designed the type face used in a particular item.
   */
  TYPE_DESIGNER,

  /**
   * Use for a person or organization primarily responsible for choice and arrangement of type used in an item. If the typographer is also responsible for other aspects of the graphic design of a book (e.g., Book designer [bkd]), codes for both functions may be needed.
   */
  TYPOGRAPHER,

  /**
   * Use for the name of a place where a university that is associated with a resource is located, for example, a university where an academic dissertation or thesis was presented.
   */
  UNIVERSITY_PLACE,

  /**
   * Use for a person or organization in charge of a video production, e.g. the video recording of a stage production as opposed to a commercial motion picture. The videographer may be the camera operator or may supervise one or more camera operators. Do not confuse with cinematographer.
   */
  VIDEOGRAPHER,

  /**
   * Use for a person or organization who principally exhibits singing skills in a musical or dramatic presentation or entertainment.
   */
  VOCALIST,

  /**
   * Use for a person who verifies the truthfulness of an event or action.
   */
  WITNESS,

  /**
   * Use for a person or organization who makes prints by cutting the image in relief on the end-grain of a wood block.
   */
  WOOD_ENGRAVER,

  /**
   * Use for a person or organization who makes prints by cutting the image in relief on the plank side of a wood block.
   */
  WOODCUTTER,

  /**
   * Use for a person or organization who writes significant material which accompanies a sound recording or other audiovisual material.
   */
  WRITER_OF_ACCOMPANYING_MATERIAL,
}

impl Relator {

  pub fn code(&self) -> &'static str {
    match self {
      Relator::ACTOR => "act",
      Relator::ADAPTER => "adp",
      Relator::ANALYST => "anl",
      Relator::ANIMATOR => "anm",
      Relator::ANNOTATOR => "ann",
      Relator::APPLICANT => "app",
      Relator::ARCHITECT => "arc",
      Relator::ARRANGER => "arr",
      Relator::ART_COPYIST => "acp",
      Relator::ARTIST => "art",
      Relator::ARTISTIC_DIRECTOR => "ard",
      Relator::ASSIGNEE => "asg",
      Relator::ASSOCIATED_NAME => "asn",
      Relator::ATTRIBUTED_NAME => "att",
      Relator::AUCTIONEER => "auc",
      Relator::AUTHOR => "aut",
      Relator::AUTHOR_IN_QUOTATIONS_OR_TEXT_EXTRACTS => "aqt",
      Relator::AUTHOR_OF_AFTERWORD_COLOPHON_ETC => "aft",
      Relator::AUTHOR_OF_DIALOG => "aud",
      Relator::AUTHOR_OF_INTRODUCTION_ETC => "aui",
      Relator::AUTHOR_OF_SCREENPLAY_ETC => "aus",
      Relator::BIBLIOGRAPHIC_ANTECEDENT => "ant",
      Relator::BINDER => "bnd",
      Relator::BINDING_DESIGNER => "bdd",
      Relator::BLURB_WRITER => "blw",
      Relator::BOOK_DESIGNER => "bkd",
      Relator::BOOK_PRODUCER => "bkp",
      Relator::BOOKJACKET_DESIGNER => "bjd",
      Relator::BOOKPLATE_DESIGNER => "bpd",
      Relator::BOOKSELLER => "bsl",
      Relator::CALLIGRAPHER => "cll",
      Relator::CARTOGRAPHER => "ctg",
      Relator::CENSOR => "cns",
      Relator::CHOREOGRAPHER => "chr",
      Relator::CINEMATOGRAPHER => "cng",
      Relator::CLIENT => "cli",
      Relator::COLLABORATOR => "clb",
      Relator::COLLECTOR => "col",
      Relator::COLLOTYPER => "clt",
      Relator::COLORIST => "clr",
      Relator::COMMENTATOR => "cmm",
      Relator::COMMENTATOR_FOR_WRITTEN_TEXT => "cwt",
      Relator::COMPILER => "com",
      Relator::COMPLAINANT => "cpl",
      Relator::COMPLAINANT_APPELLANT => "cpt",
      Relator::COMPLAINANT_APPELLEE => "cpe",
      Relator::COMPOSER => "cmp",
      Relator::COMPOSITOR => "cmt",
      Relator::CONCEPTOR => "ccp",
      Relator::CONDUCTOR => "cnd",
      Relator::CONSERVATOR => "con",
      Relator::CONSULTANT => "csl",
      Relator::CONSULTANT_TO_A_PROJECT => "csp",
      Relator::CONTESTANT => "cos",
      Relator::CONTESTANT_APPELLANT => "cot",
      Relator::CONTESTANT_APPELLEE => "coe",
      Relator::CONTESTEE => "cts",
      Relator::CONTESTEE_APPELLANT => "ctt",
      Relator::CONTESTEE_APPELLEE => "cte",
      Relator::CONTRACTOR => "ctr",
      Relator::CONTRIBUTOR => "ctb",
      Relator::COPYRIGHT_CLAIMANT => "cpc",
      Relator::COPYRIGHT_HOLDER => "cph",
      Relator::CORRECTOR => "crr",
      Relator::CORRESPONDENT => "crp",
      Relator::COSTUME_DESIGNER => "cst",
      Relator::COVER_DESIGNER => "cov",
      Relator::CREATOR => "cre",
      Relator::CURATOR_OF_AN_EXHIBITION => "cur",
      Relator::DANCER => "dnc",
      Relator::DATA_CONTRIBUTOR => "dtc",
      Relator::DATA_MANAGER => "dtm",
      Relator::DEDICATEE => "dte",
      Relator::DEDICATOR => "dto",
      Relator::DEFENDANT => "dfd",
      Relator::DEFENDANT_APPELLANT => "dft",
      Relator::DEFENDANT_APPELLEE => "dfe",
      Relator::DEGREE_GRANTOR => "dgg",
      Relator::DELINEATOR => "dln",
      Relator::DEPICTED => "dpc",
      Relator::DEPOSITOR => "dpt",
      Relator::DESIGNER => "dsr",
      Relator::DIRECTOR => "drt",
      Relator::DISSERTANT => "dis",
      Relator::DISTRIBUTION_PLACE => "dbp",
      Relator::DISTRIBUTOR => "dst",
      Relator::DONOR => "dnr",
      Relator::DRAFTSMAN => "drm",
      Relator::DUBIOUS_AUTHOR => "dub",
      Relator::EDITOR => "edt",
      Relator::ELECTRICIAN => "elg",
      Relator::ELECTROTYPER => "elt",
      Relator::ENGINEER => "eng",
      Relator::ENGRAVER => "egr",
      Relator::ETCHER => "etr",
      Relator::EVENT_PLACE => "evp",
      Relator::EXPERT => "exp",
      Relator::FACSIMILIST => "fac",
      Relator::FIELD_DIRECTOR => "fld",
      Relator::FILM_EDITOR => "flm",
      Relator::FIRST_PARTY => "fpy",
      Relator::FORGER => "frg",
      Relator::FORMER_OWNER => "fmo",
      Relator::FUNDER => "fnd",
      Relator::GEOGRAPHIC_INFORMATION_SPECIALIST => "gis",
      Relator::HONOREE => "hnr",
      Relator::HOST => "hst",
      Relator::ILLUMINATOR => "ilu",
      Relator::ILLUSTRATOR => "ill",
      Relator::INSCRIBER => "ins",
      Relator::INSTRUMENTALIST => "itr",
      Relator::INTERVIEWEE => "ive",
      Relator::INTERVIEWER => "ivr",
      Relator::INVENTOR => "inv",
      Relator::LABORATORY => "lbr",
      Relator::LABORATORY_DIRECTOR => "ldr",
      Relator::LANDSCAPE_ARCHITECT => "lsa",
      Relator::LEAD => "led",
      Relator::LENDER => "len",
      Relator::LIBELANT => "lil",
      Relator::LIBELANT_APPELLANT => "lit",
      Relator::LIBELANT_APPELLEE => "lie",
      Relator::LIBELEE => "lel",
      Relator::LIBELEE_APPELLANT => "let",
      Relator::LIBELEE_APPELLEE => "lee",
      Relator::LIBRETTIST => "lbt",
      Relator::LICENSEE => "lse",
      Relator::LICENSOR => "lso",
      Relator::LIGHTING_DESIGNER => "lgd",
      Relator::LITHOGRAPHER => "ltg",
      Relator::LYRICIST => "lyr",
      Relator::MANUFACTURER => "mfr",
      Relator::MARBLER => "mrb",
      Relator::MARKUP_EDITOR => "mrk",
      Relator::METADATA_CONTACT => "mdc",
      Relator::METAL_ENGRAVER => "mte",
      Relator::MODERATOR => "mod",
      Relator::MONITOR => "mon",
      Relator::MUSIC_COPYIST => "mcp",
      Relator::MUSICAL_DIRECTOR => "msd",
      Relator::MUSICIAN => "mus",
      Relator::NARRATOR => "nrt",
      Relator::OPPONENT => "opn",
      Relator::ORGANIZER_OF_MEETING => "orm",
      Relator::ORIGINATOR => "org",
      Relator::OTHER => "oth",
      Relator::OWNER => "own",
      Relator::PAPERMAKER => "ppm",
      Relator::PATENT_APPLICANT => "pta",
      Relator::PATENT_HOLDER => "pth",
      Relator::PATRON => "pat",
      Relator::PERFORMER => "prf",
      Relator::PERMITTING_AGENCY => "pma",
      Relator::PHOTOGRAPHER => "pht",
      Relator::PLAINTIFF => "ptf",
      Relator::PLAINTIFF_APPELLANT => "ptt",
      Relator::PLAINTIFF_APPELLEE => "pte",
      Relator::PLATEMAKER => "plt",
      Relator::PRINTER => "prt",
      Relator::PRINTER_OF_PLATES => "pop",
      Relator::PRINTMAKER => "prm",
      Relator::PROCESS_CONTACT => "prc",
      Relator::PRODUCER => "pro",
      Relator::PRODUCTION_MANAGER => "pmn",
      Relator::PRODUCTION_PERSONNEL => "prd",
      Relator::PROGRAMMER => "prg",
      Relator::PROJECT_DIRECTOR => "pdr",
      Relator::PROOFREADER => "pfr",
      Relator::PUBLICATION_PLACE => "pup",
      Relator::PUBLISHER => "pbl",
      Relator::PUBLISHING_DIRECTOR => "pbd",
      Relator::PUPPETEER => "ppt",
      Relator::RECIPIENT => "rcp",
      Relator::RECORDING_ENGINEER => "rce",
      Relator::REDACTOR => "red",
      Relator::RENDERER => "ren",
      Relator::REPORTER => "rpt",
      Relator::REPOSITORY => "rps",
      Relator::RESEARCH_TEAM_HEAD => "rth",
      Relator::RESEARCH_TEAM_MEMBER => "rtm",
      Relator::RESEARCHER => "res",
      Relator::RESPONDENT => "rsp",
      Relator::RESPONDENT_APPELLANT => "rst",
      Relator::RESPONDENT_APPELLEE => "rse",
      Relator::RESPONSIBLE_PARTY => "rpy",
      Relator::RESTAGER => "rsg",
      Relator::REVIEWER => "rev",
      Relator::RUBRICATOR => "rbr",
      Relator::SCENARIST => "sce",
      Relator::SCIENTIFIC_ADVISOR => "sad",
      Relator::SCRIBE => "scr",
      Relator::SCULPTOR => "scl",
      Relator::SECOND_PARTY => "spy",
      Relator::SECRETARY => "sec",
      Relator::SET_DESIGNER => "std",
      Relator::SIGNER => "sgn",
      Relator::SINGER => "sng",
      Relator::SOUND_DESIGNER => "sds",
      Relator::SPEAKER => "spk",
      Relator::SPONSOR => "spn",
      Relator::STAGE_MANAGER => "stm",
      Relator::STANDARDS_BODY => "stn",
      Relator::STEREOTYPER => "str",
      Relator::STORYTELLER => "stl",
      Relator::SUPPORTING_HOST => "sht",
      Relator::SURVEYOR => "srv",
      Relator::TEACHER => "tch",
      Relator::TECHNICAL_DIRECTOR => "tcd",
      Relator::THESIS_ADVISOR => "ths",
      Relator::TRANSCRIBER => "trc",
      Relator::TRANSLATOR => "trl",
      Relator::TYPE_DESIGNER => "tyd",
      Relator::TYPOGRAPHER => "tyg",
      Relator::UNIVERSITY_PLACE => "uvp",
      Relator::VIDEOGRAPHER => "vdg",
      Relator::VOCALIST => "voc",
      Relator::WITNESS => "wit",
      Relator::WOOD_ENGRAVER => "wde",
      Relator::WOODCUTTER => "wdc",
      Relator::WRITER_OF_ACCOMPANYING_MATERIAL => "wam",
    }
  }

  pub fn name(&self) -> &'static str {
    match self {
      Relator::ACTOR => "Actor",
      Relator::ADAPTER => "Adapter",
      Relator::ANALYST => "Analyst",
      Relator::ANIMATOR => "Animator",
      Relator::ANNOTATOR => "Annotator",
      Relator::APPLICANT => "Applicant",
      Relator::ARCHITECT => "Architect",
      Relator::ARRANGER => "Arranger",
      Relator::ART_COPYIST => "Art copyist",
      Relator::ARTIST => "Artist",
      Relator::ARTISTIC_DIRECTOR => "Artistic director",
      Relator::ASSIGNEE => "Assignee",
      Relator::ASSOCIATED_NAME => "Associated name",
      Relator::ATTRIBUTED_NAME => "Attributed name",
      Relator::AUCTIONEER => "Auctioneer",
      Relator::AUTHOR => "Author",
      Relator::AUTHOR_IN_QUOTATIONS_OR_TEXT_EXTRACTS => "Author in quotations or text extracts",
      Relator::AUTHOR_OF_AFTERWORD_COLOPHON_ETC => "Author of afterword, colophon, etc.",
      Relator::AUTHOR_OF_DIALOG => "Author of dialog",
      Relator::AUTHOR_OF_INTRODUCTION_ETC => "Author of introduction, etc.",
      Relator::AUTHOR_OF_SCREENPLAY_ETC => "Author of screenplay, etc.",
      Relator::BIBLIOGRAPHIC_ANTECEDENT => "Bibliographic antecedent",
      Relator::BINDER => "Binder",
      Relator::BINDING_DESIGNER => "Binding designer",
      Relator::BLURB_WRITER => "Blurb writer",
      Relator::BOOK_DESIGNER => "Book designer",
      Relator::BOOK_PRODUCER => "Book producer",
      Relator::BOOKJACKET_DESIGNER => "Bookjacket designer",
      Relator::BOOKPLATE_DESIGNER => "Bookplate designer",
      Relator::BOOKSELLER => "Bookseller",
      Relator::CALLIGRAPHER => "Calligrapher",
      Relator::CARTOGRAPHER => "Cartographer",
      Relator::CENSOR => "Censor",
      Relator::CHOREOGRAPHER => "Choreographer",
      Relator::CINEMATOGRAPHER => "Cinematographer",
      Relator::CLIENT => "Client",
      Relator::COLLABORATOR => "Collaborator",
      Relator::COLLECTOR => "Collector",
      Relator::COLLOTYPER => "Collotyper",
      Relator::COLORIST => "Colorist",
      Relator::COMMENTATOR => "Commentator",
      Relator::COMMENTATOR_FOR_WRITTEN_TEXT => "Commentator for written text",
      Relator::COMPILER => "Compiler",
      Relator::COMPLAINANT => "Complainant",
      Relator::COMPLAINANT_APPELLANT => "Complainant-appellant",
      Relator::COMPLAINANT_APPELLEE => "Complainant-appellee",
      Relator::COMPOSER => "Composer",
      Relator::COMPOSITOR => "Compositor",
      Relator::CONCEPTOR => "Conceptor",
      Relator::CONDUCTOR => "Conductor",
      Relator::CONSERVATOR => "Conservator",
      Relator::CONSULTANT => "Consultant",
      Relator::CONSULTANT_TO_A_PROJECT => "Consultant to a project",
      Relator::CONTESTANT => "Contestant",
      Relator::CONTESTANT_APPELLANT => "Contestant-appellant",
      Relator::CONTESTANT_APPELLEE => "Contestant-appellee",
      Relator::CONTESTEE => "Contestee",
      Relator::CONTESTEE_APPELLANT => "Contestee-appellant",
      Relator::CONTESTEE_APPELLEE => "Contestee-appellee",
      Relator::CONTRACTOR => "Contractor",
      Relator::CONTRIBUTOR => "Contributor",
      Relator::COPYRIGHT_CLAIMANT => "Copyright claimant",
      Relator::COPYRIGHT_HOLDER => "Copyright holder",
      Relator::CORRECTOR => "Corrector",
      Relator::CORRESPONDENT => "Correspondent",
      Relator::COSTUME_DESIGNER => "Costume designer",
      Relator::COVER_DESIGNER => "Cover designer",
      Relator::CREATOR => "Creator",
      Relator::CURATOR_OF_AN_EXHIBITION => "Curator of an exhibition",
      Relator::DANCER => "Dancer",
      Relator::DATA_CONTRIBUTOR => "Data contributor",
      Relator::DATA_MANAGER => "Data manager",
      Relator::DEDICATEE => "Dedicatee",
      Relator::DEDICATOR => "Dedicator",
      Relator::DEFENDANT => "Defendant",
      Relator::DEFENDANT_APPELLANT => "Defendant-appellant",
      Relator::DEFENDANT_APPELLEE => "Defendant-appellee",
      Relator::DEGREE_GRANTOR => "Degree grantor",
      Relator::DELINEATOR => "Delineator",
      Relator::DEPICTED => "Depicted",
      Relator::DEPOSITOR => "Depositor",
      Relator::DESIGNER => "Designer",
      Relator::DIRECTOR => "Director",
      Relator::DISSERTANT => "Dissertant",
      Relator::DISTRIBUTION_PLACE => "Distribution place",
      Relator::DISTRIBUTOR => "Distributor",
      Relator::DONOR => "Donor",
      Relator::DRAFTSMAN => "Draftsman",
      Relator::DUBIOUS_AUTHOR => "Dubious author",
      Relator::EDITOR => "Editor",
      Relator::ELECTRICIAN => "Electrician",
      Relator::ELECTROTYPER => "Electrotyper",
      Relator::ENGINEER => "Engineer",
      Relator::ENGRAVER => "Engraver",
      Relator::ETCHER => "Etcher",
      Relator::EVENT_PLACE => "Event place",
      Relator::EXPERT => "Expert",
      Relator::FACSIMILIST => "Facsimilist",
      Relator::FIELD_DIRECTOR => "Field director",
      Relator::FILM_EDITOR => "Film editor",
      Relator::FIRST_PARTY => "First party",
      Relator::FORGER => "Forger",
      Relator::FORMER_OWNER => "Former owner",
      Relator::FUNDER => "Funder",
      Relator::GEOGRAPHIC_INFORMATION_SPECIALIST => "Geographic information specialist",
      Relator::HONOREE => "Honoree",
      Relator::HOST => "Host",
      Relator::ILLUMINATOR => "Illuminator",
      Relator::ILLUSTRATOR => "Illustrator",
      Relator::INSCRIBER => "Inscriber",
      Relator::INSTRUMENTALIST => "Instrumentalist",
      Relator::INTERVIEWEE => "Interviewee",
      Relator::INTERVIEWER => "Interviewer",
      Relator::INVENTOR => "Inventor",
      Relator::LABORATORY => "Laboratory",
      Relator::LABORATORY_DIRECTOR => "Laboratory director",
      Relator::LANDSCAPE_ARCHITECT => "Landscape architect",
      Relator::LEAD => "Lead",
      Relator::LENDER => "Lender",
      Relator::LIBELANT => "Libelant",
      Relator::LIBELANT_APPELLANT => "Libelant-appellant",
      Relator::LIBELANT_APPELLEE => "Libelant-appellee",
      Relator::LIBELEE => "Libelee",
      Relator::LIBELEE_APPELLANT => "Libelee-appellant",
      Relator::LIBELEE_APPELLEE => "Libelee-appellee",
      Relator::LIBRETTIST => "Librettist",
      Relator::LICENSEE => "Licensee",
      Relator::LICENSOR => "Licensor",
      Relator::LIGHTING_DESIGNER => "Lighting designer",
      Relator::LITHOGRAPHER => "Lithographer",
      Relator::LYRICIST => "Lyricist",
      Relator::MANUFACTURER => "Manufacturer",
      Relator::MARBLER => "Marbler",
      Relator::MARKUP_EDITOR => "Markup editor",
      Relator::METADATA_CONTACT => "Metadata contact",
      Relator::METAL_ENGRAVER => "Metal-engraver",
      Relator::MODERATOR => "Moderator",
      Relator::MONITOR => "Monitor",
      Relator::MUSIC_COPYIST => "Music copyist",
      Relator::MUSICAL_DIRECTOR => "Musical director",
      Relator::MUSICIAN => "Musician",
      Relator::NARRATOR => "Narrator",
      Relator::OPPONENT => "Opponent",
      Relator::ORGANIZER_OF_MEETING => "Organizer of meeting",
      Relator::ORIGINATOR => "Originator",
      Relator::OTHER => "Other",
      Relator::OWNER => "Owner",
      Relator::PAPERMAKER => "Papermaker",
      Relator::PATENT_APPLICANT => "Patent applicant",
      Relator::PATENT_HOLDER => "Patent holder",
      Relator::PATRON => "Patron",
      Relator::PERFORMER => "Performer",
      Relator::PERMITTING_AGENCY => "Permitting agency",
      Relator::PHOTOGRAPHER => "Photographer",
      Relator::PLAINTIFF => "Plaintiff",
      Relator::PLAINTIFF_APPELLANT => "Plaintiff-appellant",
      Relator::PLAINTIFF_APPELLEE => "Plaintiff-appellee",
      Relator::PLATEMAKER => "Platemaker",
      Relator::PRINTER => "Printer",
      Relator::PRINTER_OF_PLATES => "Printer of plates",
      Relator::PRINTMAKER => "Printmaker",
      Relator::PROCESS_CONTACT => "Process contact",
      Relator::PRODUCER => "Producer",
      Relator::PRODUCTION_MANAGER => "Production manager",
      Relator::PRODUCTION_PERSONNEL => "Production personnel",
      Relator::PROGRAMMER => "Programmer",
      Relator::PROJECT_DIRECTOR => "Project director",
      Relator::PROOFREADER => "Proofreader",
      Relator::PUBLICATION_PLACE => "Publication place",
      Relator::PUBLISHER => "Publisher",
      Relator::PUBLISHING_DIRECTOR => "Publishing director",
      Relator::PUPPETEER => "Puppeteer",
      Relator::RECIPIENT => "Recipient",
      Relator::RECORDING_ENGINEER => "Recording engineer",
      Relator::REDACTOR => "Redactor",
      Relator::RENDERER => "Renderer",
      Relator::REPORTER => "Reporter",
      Relator::REPOSITORY => "Repository",
      Relator::RESEARCH_TEAM_HEAD => "Research team head",
      Relator::RESEARCH_TEAM_MEMBER => "Research team member",
      Relator::RESEARCHER => "Researcher",
      Relator::RESPONDENT => "Respondent",
      Relator::RESPONDENT_APPELLANT => "Respondent-appellant",
      Relator::RESPONDENT_APPELLEE => "Respondent-appellee",
      Relator::RESPONSIBLE_PARTY => "Responsible party",
      Relator::RESTAGER => "Restager",
      Relator::REVIEWER => "Reviewer",
      Relator::RUBRICATOR => "Rubricator",
      Relator::SCENARIST => "Scenarist",
      Relator::SCIENTIFIC_ADVISOR => "Scientific advisor",
      Relator::SCRIBE => "Scribe",
      Relator::SCULPTOR => "Sculptor",
      Relator::SECOND_PARTY => "Second party",
      Relator::SECRETARY => "Secretary",
      Relator::SET_DESIGNER => "Set designer",
      Relator::SIGNER => "Signer",
      Relator::SINGER => "Singer",
      Relator::SOUND_DESIGNER => "Sound designer",
      Relator::SPEAKER => "Speaker",
      Relator::SPONSOR => "Sponsor",
      Relator::STAGE_MANAGER => "Stage manager",
      Relator::STANDARDS_BODY => "Standards body",
      Relator::STEREOTYPER => "Stereotyper",
      Relator::STORYTELLER => "Storyteller",
      Relator::SUPPORTING_HOST => "Supporting host",
      Relator::SURVEYOR => "Surveyor",
      Relator::TEACHER => "Teacher",
      Relator::TECHNICAL_DIRECTOR => "Technical director",
      Relator::THESIS_ADVISOR => "Thesis advisor",
      Relator::TRANSCRIBER => "Transcriber",
      Relator::TRANSLATOR => "Translator",
      Relator::TYPE_DESIGNER => "Type designer",
      Relator::TYPOGRAPHER => "Typographer",
      Relator::UNIVERSITY_PLACE => "University place",
      Relator::VIDEOGRAPHER => "Videographer",
      Relator::VOCALIST => "Vocalist",
      Relator::WITNESS => "Witness",
      Relator::WOOD_ENGRAVER => "Wood-engraver",
      Relator::WOODCUTTER => "Woodcutter",
      Relator::WRITER_OF_ACCOMPANYING_MATERIAL => "Writer of accompanying material",
    }
  }

  pub fn get_code(&self) -> &'static str {
    return self.code();
  }

  pub fn get_name(&self) -> &'static str {
    return self.name();
  }

  pub fn by_code(code: &String) -> Option<Relator> {
    for relator in Relator::all_values() {
      if relator.get_code().eq_ignore_ascii_case(code) {
        return Some(relator);
      }
    }
    return None;
  }

  pub fn all_values() -> Vec<Relator> {
    vec![
      Relator::ACTOR,
      Relator::ADAPTER,
      Relator::ANALYST,
      Relator::ANIMATOR,
      Relator::ANNOTATOR,
      Relator::APPLICANT,
      Relator::ARCHITECT,
      Relator::ARRANGER,
      Relator::ART_COPYIST,
      Relator::ARTIST,
      Relator::ARTISTIC_DIRECTOR,
      Relator::ASSIGNEE,
      Relator::ASSOCIATED_NAME,
      Relator::ATTRIBUTED_NAME,
      Relator::AUCTIONEER,
      Relator::AUTHOR,
      Relator::AUTHOR_IN_QUOTATIONS_OR_TEXT_EXTRACTS,
      Relator::AUTHOR_OF_AFTERWORD_COLOPHON_ETC,
      Relator::AUTHOR_OF_DIALOG,
      Relator::AUTHOR_OF_INTRODUCTION_ETC,
      Relator::AUTHOR_OF_SCREENPLAY_ETC,
      Relator::BIBLIOGRAPHIC_ANTECEDENT,
      Relator::BINDER,
      Relator::BINDING_DESIGNER,
      Relator::BLURB_WRITER,
      Relator::BOOK_DESIGNER,
      Relator::BOOK_PRODUCER,
      Relator::BOOKJACKET_DESIGNER,
      Relator::BOOKPLATE_DESIGNER,
      Relator::BOOKSELLER,
      Relator::CALLIGRAPHER,
      Relator::CARTOGRAPHER,
      Relator::CENSOR,
      Relator::CHOREOGRAPHER,
      Relator::CINEMATOGRAPHER,
      Relator::CLIENT,
      Relator::COLLABORATOR,
      Relator::COLLECTOR,
      Relator::COLLOTYPER,
      Relator::COLORIST,
      Relator::COMMENTATOR,
      Relator::COMMENTATOR_FOR_WRITTEN_TEXT,
      Relator::COMPILER,
      Relator::COMPLAINANT,
      Relator::COMPLAINANT_APPELLANT,
      Relator::COMPLAINANT_APPELLEE,
      Relator::COMPOSER,
      Relator::COMPOSITOR,
      Relator::CONCEPTOR,
      Relator::CONDUCTOR,
      Relator::CONSERVATOR,
      Relator::CONSULTANT,
      Relator::CONSULTANT_TO_A_PROJECT,
      Relator::CONTESTANT,
      Relator::CONTESTANT_APPELLANT,
      Relator::CONTESTANT_APPELLEE,
      Relator::CONTESTEE,
      Relator::CONTESTEE_APPELLANT,
      Relator::CONTESTEE_APPELLEE,
      Relator::CONTRACTOR,
      Relator::CONTRIBUTOR,
      Relator::COPYRIGHT_CLAIMANT,
      Relator::COPYRIGHT_HOLDER,
      Relator::CORRECTOR,
      Relator::CORRESPONDENT,
      Relator::COSTUME_DESIGNER,
      Relator::COVER_DESIGNER,
      Relator::CREATOR,
      Relator::CURATOR_OF_AN_EXHIBITION,
      Relator::DANCER,
      Relator::DATA_CONTRIBUTOR,
      Relator::DATA_MANAGER,
      Relator::DEDICATEE,
      Relator::DEDICATOR,
      Relator::DEFENDANT,
      Relator::DEFENDANT_APPELLANT,
      Relator::DEFENDANT_APPELLEE,
      Relator::DEGREE_GRANTOR,
      Relator::DELINEATOR,
      Relator::DEPICTED,
      Relator::DEPOSITOR,
      Relator::DESIGNER,
      Relator::DIRECTOR,
      Relator::DISSERTANT,
      Relator::DISTRIBUTION_PLACE,
      Relator::DISTRIBUTOR,
      Relator::DONOR,
      Relator::DRAFTSMAN,
      Relator::DUBIOUS_AUTHOR,
      Relator::EDITOR,
      Relator::ELECTRICIAN,
      Relator::ELECTROTYPER,
      Relator::ENGINEER,
      Relator::ENGRAVER,
      Relator::ETCHER,
      Relator::EVENT_PLACE,
      Relator::EXPERT,
      Relator::FACSIMILIST,
      Relator::FIELD_DIRECTOR,
      Relator::FILM_EDITOR,
      Relator::FIRST_PARTY,
      Relator::FORGER,
      Relator::FORMER_OWNER,
      Relator::FUNDER,
      Relator::GEOGRAPHIC_INFORMATION_SPECIALIST,
      Relator::HONOREE,
      Relator::HOST,
      Relator::ILLUMINATOR,
      Relator::ILLUSTRATOR,
      Relator::INSCRIBER,
      Relator::INSTRUMENTALIST,
      Relator::INTERVIEWEE,
      Relator::INTERVIEWER,
      Relator::INVENTOR,
      Relator::LABORATORY,
      Relator::LABORATORY_DIRECTOR,
      Relator::LANDSCAPE_ARCHITECT,
      Relator::LEAD,
      Relator::LENDER,
      Relator::LIBELANT,
      Relator::LIBELANT_APPELLANT,
      Relator::LIBELANT_APPELLEE,
      Relator::LIBELEE,
      Relator::LIBELEE_APPELLANT,
      Relator::LIBELEE_APPELLEE,
      Relator::LIBRETTIST,
      Relator::LICENSEE,
      Relator::LICENSOR,
      Relator::LIGHTING_DESIGNER,
      Relator::LITHOGRAPHER,
      Relator::LYRICIST,
      Relator::MANUFACTURER,
      Relator::MARBLER,
      Relator::MARKUP_EDITOR,
      Relator::METADATA_CONTACT,
      Relator::METAL_ENGRAVER,
      Relator::MODERATOR,
      Relator::MONITOR,
      Relator::MUSIC_COPYIST,
      Relator::MUSICAL_DIRECTOR,
      Relator::MUSICIAN,
      Relator::NARRATOR,
      Relator::OPPONENT,
      Relator::ORGANIZER_OF_MEETING,
      Relator::ORIGINATOR,
      Relator::OTHER,
      Relator::OWNER,
      Relator::PAPERMAKER,
      Relator::PATENT_APPLICANT,
      Relator::PATENT_HOLDER,
      Relator::PATRON,
      Relator::PERFORMER,
      Relator::PERMITTING_AGENCY,
      Relator::PHOTOGRAPHER,
      Relator::PLAINTIFF,
      Relator::PLAINTIFF_APPELLANT,
      Relator::PLAINTIFF_APPELLEE,
      Relator::PLATEMAKER,
      Relator::PRINTER,
      Relator::PRINTER_OF_PLATES,
      Relator::PRINTMAKER,
      Relator::PROCESS_CONTACT,
      Relator::PRODUCER,
      Relator::PRODUCTION_MANAGER,
      Relator::PRODUCTION_PERSONNEL,
      Relator::PROGRAMMER,
      Relator::PROJECT_DIRECTOR,
      Relator::PROOFREADER,
      Relator::PUBLICATION_PLACE,
      Relator::PUBLISHER,
      Relator::PUBLISHING_DIRECTOR,
      Relator::PUPPETEER,
      Relator::RECIPIENT,
      Relator::RECORDING_ENGINEER,
      Relator::REDACTOR,
      Relator::RENDERER,
      Relator::REPORTER,
      Relator::REPOSITORY,
      Relator::RESEARCH_TEAM_HEAD,
      Relator::RESEARCH_TEAM_MEMBER,
      Relator::RESEARCHER,
      Relator::RESPONDENT,
      Relator::RESPONDENT_APPELLANT,
      Relator::RESPONDENT_APPELLEE,
      Relator::RESPONSIBLE_PARTY,
      Relator::RESTAGER,
      Relator::REVIEWER,
      Relator::RUBRICATOR,
      Relator::SCENARIST,
      Relator::SCIENTIFIC_ADVISOR,
      Relator::SCRIBE,
      Relator::SCULPTOR,
      Relator::SECOND_PARTY,
      Relator::SECRETARY,
      Relator::SET_DESIGNER,
      Relator::SIGNER,
      Relator::SINGER,
      Relator::SOUND_DESIGNER,
      Relator::SPEAKER,
      Relator::SPONSOR,
      Relator::STAGE_MANAGER,
      Relator::STANDARDS_BODY,
      Relator::STEREOTYPER,
      Relator::STORYTELLER,
      Relator::SUPPORTING_HOST,
      Relator::SURVEYOR,
      Relator::TEACHER,
      Relator::TECHNICAL_DIRECTOR,
      Relator::THESIS_ADVISOR,
      Relator::TRANSCRIBER,
      Relator::TRANSLATOR,
      Relator::TYPE_DESIGNER,
      Relator::TYPOGRAPHER,
      Relator::UNIVERSITY_PLACE,
      Relator::VIDEOGRAPHER,
      Relator::VOCALIST,
      Relator::WITNESS,
      Relator::WOOD_ENGRAVER,
      Relator::WOODCUTTER,
      Relator::WRITER_OF_ACCOMPANYING_MATERIAL,
    ]
  }

}
