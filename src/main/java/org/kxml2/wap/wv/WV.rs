// package org.kxml2.wap.wv;

// import java.io.IOException;

// import org.kxml2.wap.*;

/*

 * WV.java

 *

 * Created on 25 September 2003, 10:40

 */




   /**
     *    Wireless Village CSP 1.1 ("OMA-WV-CSP-V1_1-20021001-A.pdf")
     *    Wireless Village CSP 1.2 ("OMA-IMPS-WV-CSP_WBXML-v1_2-20030221-C.PDF")
     *    There are some bugs in the 1.2 spec but this is Ok. 1.2 is candidate
     *

 * @author  Bogdan Onoiu

 */

pub struct WV;

impl WV {

    pub fn create_parser() -> Result<WbxmlParser, IOException> {

        let mut parser = WbxmlParser::new();

        parser.set_tag_table(0, &WV::tag_table_page0);
        parser.set_tag_table(1, &WV::tag_table_page1);
        parser.set_tag_table(2, &WV::tag_table_page2);
        parser.set_tag_table(3, &WV::tag_table_page3);
        parser.set_tag_table(4, &WV::tag_table_page4);
        parser.set_tag_table(5, &WV::tag_table_page5);
        parser.set_tag_table(6, &WV::tag_table_page6);
        parser.set_tag_table(7, &WV::tag_table_page7);
        parser.set_tag_table(8, &WV::tag_table_page8);
        parser.set_tag_table(9, &WV::tag_table_page9);
        parser.set_tag_table(10, &WV::tag_table_pageA);

        parser.set_attr_start_table(0, &WV::attr_start_table);

        parser.set_attr_value_table(0, &WV::attr_value_table);

        return Ok(parser);
    }

}

    pub const tag_table_page0: [Option<&'static str>; 57] = [
        /* Common ... continue on Page 0x09 */
        Some("Acceptance"),     //0x00, 0x05
        Some("AddList"),        //0x00, 0x06
        Some("AddNickList"),    //0x00, 0x07
        Some("SName"),          //0x00, 0x08
        Some("WV-CSP-Message"), //0x00, 0x09
        Some("ClientID"),       //0x00, 0x0A
        Some("Code"),           //0x00, 0x0B
        Some("ContactList"),    //0x00, 0x0C
        Some("ContentData"),    //0x00, 0x0D
        Some("ContentEncoding"),//0x00, 0x0E
        Some("ContentSize"),    //0x00, 0x0F
        Some("ContentType"),    //0x00, 0x10
        Some("DateTime"),       //0x00, 0x11
        Some("Description"),    //0x00, 0x12
        Some("DetailedResult"), //0x00, 0x13
        Some("EntityList"),     //0x00, 0x14
        Some("Group"),          //0x00, 0x15
        Some("GroupID"),        //0x00, 0x16
        Some("GroupList"),      //0x00, 0x17
        Some("InUse"),          //0x00, 0x18
        Some("Logo"),           //0x00, 0x19
        Some("MessageCount"),   //0x00, 0x1A
        Some("MessageID"),      //0x00, 0x1B
        Some("MessageURI"),     //0x00, 0x1C
        Some("MSISDN"),         //0x00, 0x1D
        Some("Name"),           //0x00, 0x1E
        Some("NickList"),       //0x00, 0x1F
        Some("NickName"),       //0x00, 0x20
        Some("Poll"),           //0x00, 0x21
        Some("Presence"),       //0x00, 0x22
        Some("PresenceSubList"),//0x00, 0x23
        Some("PresenceValue"),  //0x00, 0x24
        Some("Property"),       //0x00, 0x25
        Some("Qualifier"),      //0x00, 0x26
        Some("Recipient"),      //0x00, 0x27
        Some("RemoveList"),     //0x00, 0x28
        Some("RemoveNickList"), //0x00, 0x29
        Some("Result"),         //0x00, 0x2A
        Some("ScreenName"),     //0x00, 0x2B
        Some("Sender"),         //0x00, 0x2C
        Some("Session"),        //0x00, 0x2D
        Some("SessionDescriptor"),//0x00, 0x2E
        Some("SessionID"),      //0x00, 0x2F
        Some("SessionType"),    //0x00, 0x30
        Some("Status"),         //0x00, 0x31
        Some("Transaction"),    //0x00, 0x32
        Some("TransactionContent"),//0x00, 0x33
        Some("TransactionDescriptor"),//0x00, 0x34
        Some("TransactionID"),  //0x00, 0x35
        Some("TransactionMode"),//0x00, 0x36
        Some("URL"),            //0x00, 0x37
        Some("URLList"),        //0x00, 0x38
        Some("User"),           //0x00, 0x39
        Some("UserID"),         //0x00, 0x3A
        Some("UserList"),       //0x00, 0x3B
        Some("Validity"),       //0x00, 0x3C
        Some("Value"),          //0x00, 0x3D
    ];

    pub const tag_table_page1: [Option<&'static str>; 59] = [
        /* Access ... continue on Page 0x0A */
        Some("AllFunctions"),             //  0x01, 0x05
        Some("AllFunctionsRequest"),      //  0x01, 0x06
        Some("CancelInvite-Request"),     //  0x01, 0x07
        Some("CancelInviteUser-Request"), //  0x01, 0x08
        Some("Capability"),               //  0x01, 0x09
        Some("CapabilityList"),           //  0x01, 0x0A
        Some("CapabilityRequest"),        //  0x01, 0x0B
        Some("ClientCapability-Request"), //  0x01, 0x0C
        Some("ClientCapability-Response"),//  0x01, 0x0D
        Some("DigestBytes"),          //  0x01, 0x0E
        Some("DigestSchema"),         //  0x01, 0x0F
        Some("Disconnect"),           //  0x01, 0x10
        Some("Functions"),            //  0x01, 0x11
        Some("GetSPInfo-Request"),    //  0x01, 0x12
        Some("GetSPInfo-Response"),   //  0x01, 0x13
        Some("InviteID"),             //  0x01, 0x14
        Some("InviteNote"),           //  0x01, 0x15
        Some("Invite-Request"),       //  0x01, 0x16
        Some("Invite-Response"),      //  0x01, 0x17
        Some("InviteType"),           //  0x01, 0x18
        Some("InviteUser-Request"),   //  0x01, 0x19
        Some("InviteUser-Response"),  //  0x01, 0x1A
        Some("KeepAlive-Request"),    //  0x01, 0x1B
        Some("KeepAliveTime"),        //  0x01, 0x1C
        Some("Login-Request"),        //  0x01, 0x1D
        Some("Login-Response"),       //  0x01, 0x1E
        Some("Logout-Request"),       //  0x01, 0x1F
        Some("Nonce"),                //  0x01, 0x20
        Some("Password"),             //  0x01, 0x21
        Some("Polling-Request"),      //  0x01, 0x22
        Some("ResponseNote"),         //  0x01, 0x23
        Some("SearchElement"),        //  0x01, 0x24
        Some("SearchFindings"),       //  0x01, 0x25
        Some("SearchID"),             //  0x01, 0x26
        Some("SearchIndex"),          //  0x01, 0x27
        Some("SearchLimit"),          //  0x01, 0x28
        Some("KeepAlive-Response"),   //  0x01, 0x29
        Some("SearchPairList"),       //  0x01, 0x2A
        Some("Search-Request"),       //  0x01, 0x2B
        Some("Search-Response"),      //  0x01, 0x2C
        Some("SearchResult"),         //  0x01, 0x2D
        Some("Service-Request"),      //  0x01, 0x2E
        Some("Service-Response"),     //  0x01, 0x2F
        Some("SessionCookie"),        //  0x01, 0x30
        Some("StopSearch-Request"),   //  0x01, 0x31
        Some("TimeToLive"),           //  0x01, 0x32
        Some("SearchString"),         //  0x01, 0x33
        Some("CompletionFlag"),       //  0x01, 0x34
        None,                   //  0x01, 0x35
        Some("ReceiveList"),          //  0x01, 0x36 /* WV 1.2 */
        Some("VerifyID-Request"),     //  0x01, 0x37 /* WV 1.2 */
        Some("Extended-Request"),     //  0x01, 0x38 /* WV 1.2 */
        Some("Extended-Response"),    //  0x01, 0x39 /* WV 1.2 */
        Some("AgreedCapabilityList"), //  0x01, 0x3A /* WV 1.2 */
        Some("Extended-Data"),        //  0x01, 0x3B /* WV 1.2 */
        Some("OtherServer"),          //  0x01, 0x3C /* WV 1.2 */
        Some("PresenceAttributeNSName"),//0x01, 0x3D /* WV 1.2 */
        Some("SessionNSName"),        //  0x01, 0x3E /* WV 1.2 */
        Some("TransactionNSName"),    //  0x01, 0x3F /* WV 1.2 */
    ];

    pub const tag_table_page2: [Option<&'static str>; 59] = [
        /* Service ... continue on Page 0x08 */
        Some("ADDGM"),        //  0x02, 0x05
        Some("AttListFunc"),  //  0x02, 0x06
        Some("BLENT"),        //  0x02, 0x07
        Some("CAAUT"),        //  0x02, 0x08
        Some("CAINV"),        //  0x02, 0x09
        Some("CALI"),         //  0x02, 0x0A
        Some("CCLI"),         //  0x02, 0x0B
        Some("ContListFunc"), //  0x02, 0x0C
        Some("CREAG"),        //  0x02, 0x0D
        Some("DALI"),         //  0x02, 0x0E
        Some("DCLI"),         //  0x02, 0x0F
        Some("DELGR"),        //  0x02, 0x10
        Some("FundamentalFeat"),//0x02, 0x11
        Some("FWMSG"),        //  0x02, 0x12
        Some("GALS"),         //  0x02, 0x13
        Some("GCLI"),         //  0x02, 0x14
        Some("GETGM"),        //  0x02, 0x15
        Some("GETGP"),        //  0x02, 0x16
        Some("GETLM"),        //  0x02, 0x17
        Some("GETM"),         //  0x02, 0x18
        Some("GETPR"),        //  0x02, 0x19
        Some("GETSPI"),       //  0x02, 0x1A
        Some("GETWL"),        //  0x02, 0x1B
        Some("GLBLU"),        //  0x02, 0x1C
        Some("GRCHN"),        //  0x02, 0x1D
        Some("GroupAuthFunc"),//  0x02, 0x1E
        Some("GroupFeat"),    //  0x02, 0x1F
        Some("GroupMgmtFunc"),//  0x02, 0x20
        Some("GroupUseFunc"), //  0x02, 0x21
        Some("IMAuthFunc"),   //  0x02, 0x22
        Some("IMFeat"),       //  0x02, 0x23
        Some("IMReceiveFunc"),//  0x02, 0x24
        Some("IMSendFunc"),   //  0x02, 0x25
        Some("INVIT"),        //  0x02, 0x26
        Some("InviteFunc"),   //  0x02, 0x27
        Some("MBRAC"),        //  0x02, 0x28
        Some("MCLS"),         //  0x02, 0x29
        Some("MDELIV"),       //  0x02, 0x2A
        Some("NEWM"),         //  0x02, 0x2B
        Some("NOTIF"),        //  0x02, 0x2C
        Some("PresenceAuthFunc"),//0x02, 0x2D
        Some("PresenceDeliverFunc"),//0x02, 0x2E
        Some("PresenceFeat"), //  0x02, 0x2F
        Some("REACT"),        //  0x02, 0x30
        Some("REJCM"),        //  0x02, 0x31
        Some("REJEC"),        //  0x02, 0x32
        Some("RMVGM"),        //  0x02, 0x33
        Some("SearchFunc"),   //  0x02, 0x34
        Some("ServiceFunc"),  //  0x02, 0x35
        Some("SETD"),         //  0x02, 0x36
        Some("SETGP"),        //  0x02, 0x37
        Some("SRCH"),         //  0x02, 0x38
        Some("STSRC"),        //  0x02, 0x39
        Some("SUBGCN"),       //  0x02, 0x3A
        Some("UPDPR"),        //  0x02, 0x3B
        Some("WVCSPFeat"),    //  0x02, 0x3C
        Some("MF"),           //  0x02, 0x3D /* WV 1.2 */
        Some("MG"),           //  0x02, 0x3E /* WV 1.2 */
        Some("MM")            //  0x02, 0x3F /* WV 1.2 */
    ];

    pub const tag_table_page3: [Option<&'static str>; 15] = [
        /* Client Capability */
        Some("AcceptedCharset"),          //  0x03, 0x05
        Some("AcceptedContentLength"),    //  0x03, 0x06
        Some("AcceptedContentType"),      //  0x03, 0x07
        Some("AcceptedTransferEncoding"), //  0x03, 0x08
        Some("AnyContent"),               //  0x03, 0x09
        Some("DefaultLanguage"),          //  0x03, 0x0A
        Some("InitialDeliveryMethod"),    //  0x03, 0x0B
        Some("MultiTrans"),               //  0x03, 0x0C
        Some("ParserSize"),               //  0x03, 0x0D
        Some("ServerPollMin"),            //  0x03, 0x0E
        Some("SupportedBearer"),          //  0x03, 0x0F
        Some("SupportedCIRMethod"),       //  0x03, 0x10
        Some("TCPAddress"),               //  0x03, 0x11
        Some("TCPPort"),                  //  0x03, 0x12
        Some("UDPPort")                  //  0x03, 0x13
    ];

    pub const tag_table_page4: [Option<&'static str>; 28] = [
        /* Presence Primitive */
        Some("CancelAuth-Request"),           //  0x04, 0x05
        Some("ContactListProperties"),        //  0x04, 0x06
        Some("CreateAttributeList-Request"),  //  0x04, 0x07
        Some("CreateList-Request"),           //  0x04, 0x08
        Some("DefaultAttributeList"),         //  0x04, 0x09
        Some("DefaultContactList"),           //  0x04, 0x0A
        Some("DefaultList"),                  //  0x04, 0x0B
        Some("DeleteAttributeList-Request"),  //  0x04, 0x0C
        Some("DeleteList-Request"),           //  0x04, 0x0D
        Some("GetAttributeList-Request"),     //  0x04, 0x0E
        Some("GetAttributeList-Response"),    //  0x04, 0x0F
        Some("GetList-Request"),              //  0x04, 0x10
        Some("GetList-Response"),             //  0x04, 0x11
        Some("GetPresence-Request"),          //  0x04, 0x12
        Some("GetPresence-Response"),         //  0x04, 0x13
        Some("GetWatcherList-Request"),       //  0x04, 0x14
        Some("GetWatcherList-Response"),      //  0x04, 0x15
        Some("ListManage-Request"),           //  0x04, 0x16
        Some("ListManage-Response"),          //  0x04, 0x17
        Some("UnsubscribePresence-Request"),  //  0x04, 0x18
        Some("PresenceAuth-Request"),         //  0x04, 0x19
        Some("PresenceAuth-User"),            //  0x04, 0x1A
        Some("PresenceNotification-Request"), //  0x04, 0x1B
        Some("UpdatePresence-Request"),       //  0x04, 0x1C
        Some("SubscribePresence-Request"),    //  0x04, 0x1D
        Some("Auto-Subscribe"),               //  0x04, 0x1E /* WV 1.2 */
        Some("GetReactiveAuthStatus-Request"),//  0x04, 0x1F /* WV 1.2 */
        Some("GetReactiveAuthStatus-Response"),// 0x04, 0x20 /* WV 1.2 */
    ];

    pub const tag_table_page5: [Option<&'static str>; 54] = [
        /* Presence Attribute */
        Some("Accuracy"),         //  0x05, 0x05
        Some("Address"),          //  0x05, 0x06
        Some("AddrPref"),         //  0x05, 0x07
        Some("Alias"),            //  0x05, 0x08
        Some("Altitude"),         //  0x05, 0x09
        Some("Building"),         //  0x05, 0x0A
        Some("Caddr"),            //  0x05, 0x0B
        Some("City"),             //  0x05, 0x0C
        Some("ClientInfo"),       //  0x05, 0x0D
        Some("ClientProducer"),   //  0x05, 0x0E
        Some("ClientType"),       //  0x05, 0x0F
        Some("ClientVersion"),    //  0x05, 0x10
        Some("CommC"),            //  0x05, 0x11
        Some("CommCap"),          //  0x05, 0x12
        Some("ContactInfo"),      //  0x05, 0x13
        Some("ContainedvCard"),   //  0x05, 0x14
        Some("Country"),          //  0x05, 0x15
        Some("Crossing1"),        //  0x05, 0x16
        Some("Crossing2"),        //  0x05, 0x17
        Some("DevManufacturer"),  //  0x05, 0x18
        Some("DirectContent"),    //  0x05, 0x19
        Some("FreeTextLocation"), //  0x05, 0x1A
        Some("GeoLocation"),      //  0x05, 0x1B
        Some("Language"),         //  0x05, 0x1C
        Some("Latitude"),         //  0x05, 0x1D
        Some("Longitude"),        //  0x05, 0x1E
        Some("Model"),            //  0x05, 0x1F
        Some("NamedArea"),        //  0x05, 0x20
        Some("OnlineStatus"),     //  0x05, 0x21
        Some("PLMN"),             //  0x05, 0x22
        Some("PrefC"),            //  0x05, 0x23
        Some("PreferredContacts"),//  0x05, 0x24
        Some("PreferredLanguage"),//  0x05, 0x25
        Some("PreferredContent"), //  0x05, 0x26
        Some("PreferredvCard"),   //  0x05, 0x27
        Some("Registration"),     //  0x05, 0x28
        Some("StatusContent"),    //  0x05, 0x29
        Some("StatusMood"),       //  0x05, 0x2A
        Some("StatusText"),       //  0x05, 0x2B
        Some("Street"),           //  0x05, 0x2C
        Some("TimeZone"),         //  0x05, 0x2D
        Some("UserAvailability"), //  0x05, 0x2E
        Some("Cap"),              //  0x05, 0x2F
        Some("Cname"),            //  0x05, 0x30
        Some("Contact"),          //  0x05, 0x31
        Some("Cpriority"),        //  0x05, 0x32
        Some("Cstatus"),          //  0x05, 0x33
        Some("Note"),             //  0x05, 0x34 /* WV 1.2 */
        Some("Zone"),             //  0x05, 0x35
        None,
        Some("Inf_link"),         //  0x05, 0x37 /* WV 1.2 */
        Some("InfoLink"),         //  0x05, 0x38 /* WV 1.2 */
        Some("Link"),             //  0x05, 0x39 /* WV 1.2 */
        Some("Text"),             //  0x05, 0x3A /* WV 1.2 */
    ];

    pub const tag_table_page6: [Option<&'static str>; 22] = [
        /* Messaging */
        Some("BlockList"),                //  0x06, 0x05
//      "BlockUser-Request",        //  0x06, 0x06  //This is a bug in the spec
        Some("BlockEntity-Request"),        //  0x06, 0x06
        Some("DeliveryMethod"),           //  0x06, 0x07
        Some("DeliveryReport"),           //  0x06, 0x08
        Some("DeliveryReport-Request"),   //  0x06, 0x09
        Some("ForwardMessage-Request"),   //  0x06, 0x0A
        Some("GetBlockedList-Request"),   //  0x06, 0x0B
        Some("GetBlockedList-Response"),  //  0x06, 0x0C
        Some("GetMessageList-Request"),   //  0x06, 0x0D
        Some("GetMessageList-Response"),  //  0x06, 0x0E
        Some("GetMessage-Request"),       //  0x06, 0x0F
        Some("GetMessage-Response"),      //  0x06, 0x10
        Some("GrantList"),                //  0x06, 0x11
        Some("MessageDelivered"),         //  0x06, 0x12
        Some("MessageInfo"),              //  0x06, 0x13
        Some("MessageNotification"),      //  0x06, 0x14
        Some("NewMessage"),               //  0x06, 0x15
        Some("RejectMessage-Request"),    //  0x06, 0x16
        Some("SendMessage-Request"),      //  0x06, 0x17
        Some("SendMessage-Response"),     //  0x06, 0x18
        Some("SetDeliveryMethod-Request"),//  0x06, 0x19
        Some("DeliveryTime"),             //  0x06, 0x1A
    ];

    pub const tag_table_page7: [Option<&'static str>; 39] = [
        /* Group */
        Some("AddGroupMembers-Request"),  //  0x07, 0x05
        Some("Admin"),                    //  0x07, 0x06
        Some("CreateGroup-Request"),      //  0x07, 0x07
        Some("DeleteGroup-Request"),      //  0x07, 0x08
        Some("GetGroupMembers-Request"),  //  0x07, 0x09
        Some("GetGroupMembers-Response"), //  0x07, 0x0A
        Some("GetGroupProps-Request"),    //  0x07, 0x0B
        Some("GetGroupProps-Response"),   //  0x07, 0x0C
        Some("GroupChangeNotice"),        //  0x07, 0x0D
        Some("GroupProperties"),          //  0x07, 0x0E
        Some("Joined"),                   //  0x07, 0x0F
        Some("JoinedRequest"),            //  0x07, 0x10
        Some("JoinGroup-Request"),        //  0x07, 0x11
        Some("JoinGroup-Response"),       //  0x07, 0x12
        Some("LeaveGroup-Request"),       //  0x07, 0x13
        Some("LeaveGroup-Response"),      //  0x07, 0x14
        Some("Left"),                     //  0x07, 0x15
        Some("MemberAccess-Request"),     //  0x07, 0x16
        Some("Mod"),                      //  0x07, 0x17
        Some("OwnProperties"),            //  0x07, 0x18
        Some("RejectList-Request"),       //  0x07, 0x19
        Some("RejectList-Response"),      //  0x07, 0x1A
        Some("RemoveGroupMembers-Request"),// 0x07, 0x1B
        Some("SetGroupProps-Request"),    //  0x07, 0x1C
        Some("SubscribeGroupNotice-Request"), //  0x07, 0x1D
        Some("SubscribeGroupNotice-Response"),//  0x07, 0x1E
        Some("Users"),                    //  0x07, 0x1F
        Some("WelcomeNote"),              //  0x07, 0x20
        Some("JoinGroup"),                //  0x07, 0x21
        Some("SubscribeNotification"),    //  0x07, 0x22
        Some("SubscribeType"),            //  0x07, 0x23
        Some("GetJoinedUsers-Request"),   //  0x07, 0x24 /* WV 1.2 */
        Some("GetJoinedUsers-Response"),  //  0x07, 0x25 /* WV 1.2 */
        Some("AdminMapList"),             //  0x07, 0x26 /* WV 1.2 */
        Some("AdminMapping"),             //  0x07, 0x27 /* WV 1.2 */
        Some("Mapping"),                  //  0x07, 0x28 /* WV 1.2 */
        Some("ModMapping"),               //  0x07, 0x29 /* WV 1.2 */
        Some("UserMapList"),              //  0x07, 0x2A /* WV 1.2 */
        Some("UserMapping"),              //  0x07, 0x2B /* WV 1.2 */
    ];

    pub const tag_table_page8: [Option<&'static str>; 5] = [
        /* Service ... continued */
        Some("MP"),                       //  0x08, 0x05 /* WV 1.2 */
        Some("GETAUT"),                   //  0x08, 0x06 /* WV 1.2 */
        Some("GETJU"),                    //  0x08, 0x07 /* WV 1.2 */
        Some("VRID"),                     //  0x08, 0x08 /* WV 1.2 */
        Some("VerifyIDFunc"),             //  0x08, 0x09 /* WV 1.2 */
    ];

    pub const tag_table_page9: [Option<&'static str>; 11] = [
        /* Common ... continued */
        Some("CIR"),                      //  0x09, 0x05 /* WV 1.2 */
        Some("Domain"),                   //  0x09, 0x06 /* WV 1.2 */
        Some("ExtBlock"),                 //  0x09, 0x07 /* WV 1.2 */
        Some("HistoryPeriod"),            //  0x09, 0x08 /* WV 1.2 */
        Some("IDList"),                   //  0x09, 0x09 /* WV 1.2 */
        Some("MaxWatcherList"),           //  0x09, 0x0A /* WV 1.2 */
        Some("ReactiveAuthState"),        //  0x09, 0x0B /* WV 1.2 */
        Some("ReactiveAuthStatus"),       //  0x09, 0x0C /* WV 1.2 */
        Some("ReactiveAuthStatusList"),   //  0x09, 0x0D /* WV 1.2 */
        Some("Watcher"),                  //  0x09, 0x0E /* WV 1.2 */
        Some("WatcherStatus")             //  0x09, 0x0F /* WV 1.2 */
    ];

    pub const tag_table_page_a: [Option<&'static str>; 3] = [
        /* Access ... continued */
        Some("WV-CSP-NSDiscovery-Request"),  //0x0A, 0x05 /* WV 1.2 */
        Some("WV-CSP-NSDiscovery-Response"), //0x0A, 0x06 /* WV 1.2 */
        Some("VersionList")                  //0x0A, 0x07 /* WV 1.2 */
    ];

    pub const attr_start_table: [Option<&'static str>; 6] = [
        Some("xmlns=http://www.wireless-village.org/CSP"),//  0x00, 0x05
        Some("xmlns=http://www.wireless-village.org/PA"), //  0x00, 0x06
        Some("xmlns=http://www.wireless-village.org/TRC"),//  0x00, 0x07
        Some("xmlns=http://www.openmobilealliance.org/DTD/WV-CSP"),   //  0x00, 0x08
        Some("xmlns=http://www.openmobilealliance.org/DTD/WV-PA"),    //  0x00, 0x09
        Some("xmlns=http://www.openmobilealliance.org/DTD/WV-TRC"),   //  0x00, 0x0A
    ];

    pub const attr_value_table: [Option<&'static str>; 120] = {

        Some("AccessType"),                           // 0x00 /* Common value token */
        Some("ActiveUsers"),                          // 0x01 /* Common value token */
        Some("Admin"),                                // 0x02 /* Common value token */
        Some("application/"),                         // 0x03 /* Common value token */
        Some("application/vnd.wap.mms-message"),      // 0x04 /* Common value token */
        Some("application/x-sms"),                    // 0x05 /* Common value token */
        Some("AutoJoin"),                             // 0x06 /* Common value token */
        Some("BASE64"),                               // 0x07 /* Common value token */
        Some("Closed"),                               // 0x08 /* Common value token */
        Some("Default"),                              // 0x09 /* Common value token */
        Some("DisplayName"),                          // 0x0a /* Common value token */
        Some("F"),                                    // 0x0b /* Common value token */
        Some("G"),                                    // 0x0c /* Common value token */
        Some("GR"),                                   // 0x0d /* Common value token */
        Some("http://"),                              // 0x0e /* Common value token */
        Some("https://"),                             // 0x0f /* Common value token */
        Some("image/"),                               // 0x10 /* Common value token */
        Some("Inband"),                               // 0x11 /* Common value token */
        Some("IM"),                                   // 0x12 /* Common value token */
        Some("MaxActiveUsers"),                       // 0x13 /* Common value token */
        Some("Mod"),                                  // 0x14 /* Common value token */
        Some("Name"),                                 // 0x15 /* Common value token */
        Some("None"),                                 // 0x16 /* Common value token */
        Some("N"),                                    // 0x17 /* Common value token */
        Some("Open"),                                 // 0x18 /* Common value token */
        Some("Outband"),                              // 0x19 /* Common value token */
        Some("PR"),                                   // 0x1a /* Common value token */
        Some("Private"),                              // 0x1b /* Common value token */
        Some("PrivateMessaging"),                     // 0x1c /* Common value token */
        Some("PrivilegeLevel"),                       // 0x1d /* Common value token */
        Some("Public"),                               // 0x1e /* Common value token */
        Some("P"),                                    // 0x1f /* Common value token */
        Some("Request"),                              // 0x20 /* Common value token */
        Some("Response"),                             // 0x21 /* Common value token */
        Some("Restricted"),                           // 0x22 /* Common value token */
        Some("ScreenName"),                           // 0x23 /* Common value token */
        Some("Searchable"),                           // 0x24 /* Common value token */
        Some("S"),                                    // 0x25 /* Common value token */
        Some("SC"),                                   // 0x26 /* Common value token */
        Some("text/"),                                // 0x27 /* Common value token */
        Some("text/plain"),                           // 0x28 /* Common value token */
        Some("text/x-vCalendar"),                     // 0x29 /* Common value token */
        Some("text/x-vCard"),                         // 0x2a /* Common value token */
        Some("Topic"),                                // 0x2b /* Common value token */
        Some("T"),                                    // 0x2c /* Common value token */
        Some("Type"),                                 // 0x2d /* Common value token */
        Some("U"),                                    // 0x2e /* Common value token */
        Some("US"),                                   // 0x2f /* Common value token */
        Some("www.wireless-village.org"),             // 0x30 /* Common value token */
        Some("AutoDelete"),                           // 0x31 /* Common value token */ /* WV 1.2 */
        Some("GM"),                                   // 0x32 /* Common value token */ /* WV 1.2 */
        Some("Validity"),                             // 0x33 /* Common value token */ /* WV 1.2 */
        Some("ShowID"),                               // 0x34 /* Common value token */ /* WV 1.2 */
        Some("GRANTED"),                              // 0x35 /* Common value token */ /* WV 1.2 */
        Some("PENDING"),                              // 0x36 /* Common value token */ /* WV 1.2 */
        None,                                   // 0x37
        None,                                   // 0x38
        None,                                   // 0x39
        None,                                   // 0x3a
        None,                                   // 0x3b
        None,                                   // 0x3c
        Some("GROUP_ID"),                             // 0x3d /* Access value token */
        Some("GROUP_NAME"),                           // 0x3e /* Access value token */
        Some("GROUP_TOPIC"),                          // 0x3f /* Access value token */
        Some("GROUP_USER_ID_JOINED"),                 // 0x40 /* Access value token */
        Some("GROUP_USER_ID_OWNER"),                  // 0x41 /* Access value token */
        Some("HTTP"),                                 // 0x42 /* Access value token */
        Some("SMS"),                                  // 0x43 /* Access value token */
        Some("STCP"),                                 // 0x44 /* Access value token */
        Some("SUDP"),                                 // 0x45 /* Access value token */
        Some("USER_ALIAS"),                           // 0x46 /* Access value token */
        Some("USER_EMAIL_ADDRESS"),                   // 0x47 /* Access value token */
        Some("USER_FIRST_NAME"),                      // 0x48 /* Access value token */
        Some("USER_ID"),                              // 0x49 /* Access value token */
        Some("USER_LAST_NAME"),                       // 0x4a /* Access value token */
        Some("USER_MOBILE_NUMBER"),                   // 0x4b /* Access value token */
        Some("USER_ONLINE_STATUS"),                   // 0x4c /* Access value token */
        Some("WAPSMS"),                               // 0x4d /* Access value token */
        Some("WAPUDP"),                               // 0x4e /* Access value token */
        Some("WSP"),                                  // 0x4f /* Access value token */
        Some("GROUP_USER_ID_AUTOJOIN"),               // 0x50 /* Access value token */ /* WV 1.2 */
        None,                                   // 0x51
        None,                                   // 0x52
        None,                                   // 0x53
        None,                                   // 0x54
        None,                                   // 0x55
        None,                                   // 0x56
        None,                                   // 0x57
        None,                                   // 0x58
        None,                                   // 0x59
        None,                                   // 0x5a
        Some("ANGRY"),                                // 0x5b /* Presence value token */
        Some("ANXIOUS"),                              // 0x5c /* Presence value token */
        Some("ASHAMED"),                              // 0x5d /* Presence value token */
        Some("AUDIO_CALL"),                           // 0x5e /* Presence value token */
        Some("AVAILABLE"),                            // 0x5f /* Presence value token */
        Some("BORED"),                                // 0x60 /* Presence value token */
        Some("CALL"),                                 // 0x61 /* Presence value token */
        Some("CLI"),                                  // 0x62 /* Presence value token */
        Some("COMPUTER"),                             // 0x63 /* Presence value token */
        Some("DISCREET"),                             // 0x64 /* Presence value token */
        Some("EMAIL"),                                // 0x65 /* Presence value token */
        Some("EXCITED"),                              // 0x66 /* Presence value token */
        Some("HAPPY"),                                // 0x67 /* Presence value token */
        Some("IM"),                                   // 0x68 /* Presence value token */
        Some("IM_OFFLINE"),                           // 0x69 /* Presence value token */
        Some("IM_ONLINE"),                            // 0x6a /* Presence value token */
        Some("IN_LOVE"),                              // 0x6b /* Presence value token */
        Some("INVINCIBLE"),                           // 0x6c /* Presence value token */
        Some("JEALOUS"),                              // 0x6d /* Presence value token */
        Some("MMS"),                                  // 0x6e /* Presence value token */
        Some("MOBILE_PHONE"),                         // 0x6f /* Presence value token */
        Some("NOT_AVAILABLE"),                        // 0x70 /* Presence value token */
        Some("OTHER"),                                // 0x71 /* Presence value token */
        Some("PDA"),                                  // 0x72 /* Presence value token */
        Some("SAD"),                                  // 0x73 /* Presence value token */
        Some("SLEEPY"),                               // 0x74 /* Presence value token */
        Some("SMS"),                                  // 0x75 /* Presence value token */
        Some("VIDEO_CALL"),                           // 0x76 /* Presence value token */
        Some("VIDEO_STREAM"),                         // 0x77 /* Presence value token */
    };
